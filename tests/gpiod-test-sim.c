/* SPDX-License-Identifier: LGPL-2.1-or-later */
/* SPDX-FileCopyrightText: 2022 Bartosz Golaszewski <brgl@bgdev.pl> */

#include <errno.h>
#include <gpiosim.h>
#include <stdlib.h>
#include <unistd.h>

#include "gpiod-test-sim.h"

G_DEFINE_QUARK(g-gpiosim-error, g_gpiosim_error);

struct _GPIOSimChip {
	GObject parent_instance;
	struct gpiosim_bank *bank;
	GError *construct_err;
	guint num_lines;
	gchar *label;
	GVariant *line_names;
	GVariant *hogs;
};

enum {
	G_GPIOSIM_CHIP_PROP_DEV_PATH = 1,
	G_GPIOSIM_CHIP_PROP_NAME,
	G_GPIOSIM_CHIP_PROP_NUM_LINES,
	G_GPIOSIM_CHIP_PROP_LABEL,
	G_GPIOSIM_CHIP_PROP_LINE_NAMES,
	G_GPIOSIM_CHIP_PROP_HOGS,
};

static struct gpiosim_ctx *sim_ctx;

static gboolean
g_gpiosim_chip_initable_init(GInitable *initable,
			     GCancellable *cancellable G_GNUC_UNUSED,
			     GError **err)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP_OBJ(initable);

	if (self->construct_err) {
		g_propagate_error(err, self->construct_err);
		self->construct_err = NULL;
		return FALSE;
	}

	return TRUE;
}

static void g_gpiosim_chip_initable_iface_init(GInitableIface *iface)
{
	iface->init = g_gpiosim_chip_initable_init;
}

G_DEFINE_TYPE_WITH_CODE(GPIOSimChip, g_gpiosim_chip, G_TYPE_OBJECT,
			G_IMPLEMENT_INTERFACE(
				G_TYPE_INITABLE,
				g_gpiosim_chip_initable_iface_init));

static void g_gpiosim_ctx_unref(void)
{
	gpiosim_ctx_unref(sim_ctx);
}

static gboolean g_gpiosim_chip_apply_line_names(GPIOSimChip *self)
{
	g_autoptr(GVariantIter) iter = NULL;
	guint offset;
	gchar *name;
	int ret;

	if (!self->line_names)
		return TRUE;

	iter = g_variant_iter_new(self->line_names);

	while (g_variant_iter_loop(iter, "(us)", &offset, &name)) {
		ret = gpiosim_bank_set_line_name(self->bank, offset, name);
		if (ret) {
			g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
				    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
				    "Unable to set the name of the simulated GPIO line: %s",
				    g_strerror(errno));
			return FALSE;
		}
	}

	return TRUE;
}

static gboolean g_gpiosim_chip_apply_hogs(GPIOSimChip *self)
{
	g_autoptr(GVariantIter) iter = NULL;
	enum gpiosim_direction dir;
	guint offset;
	gchar *name;
	gint vdir;
	int ret;

	if (!self->hogs)
		return TRUE;

	iter = g_variant_iter_new(self->hogs);

	while (g_variant_iter_loop(iter, "(usi)", &offset, &name, &vdir)) {
		switch (vdir) {
		case G_GPIOSIM_DIRECTION_INPUT:
			dir = GPIOSIM_DIRECTION_INPUT;
			break;
		case G_GPIOSIM_DIRECTION_OUTPUT_HIGH:
			dir = GPIOSIM_DIRECTION_OUTPUT_HIGH;
			break;
		case G_GPIOSIM_DIRECTION_OUTPUT_LOW:
			dir = GPIOSIM_DIRECTION_OUTPUT_LOW;
			break;
		default:
			g_error("Invalid hog direction value: %d", vdir);
		}

		ret = gpiosim_bank_hog_line(self->bank, offset, name, dir);
		if (ret) {
			g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
				    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
				    "Unable to hog the simulated GPIO line: %s",
				    g_strerror(errno));
			return FALSE;
		}
	}

	return TRUE;
}

static gboolean g_gpiosim_chip_apply_properties(GPIOSimChip *self)
{
	int ret;

	ret = gpiosim_bank_set_num_lines(self->bank, self->num_lines);
	if (ret) {
		g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
			    "Unable to set the number of lines exposed by the simulated chip: %s",
			    g_strerror(errno));
		return FALSE;
	}

	if (self->label) {
		ret = gpiosim_bank_set_label(self->bank, self->label);
		if (ret) {
			g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
				    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
				    "Unable to set the label of the simulated chip: %s",
				    g_strerror(errno));
			return FALSE;
		}
	}

	ret = g_gpiosim_chip_apply_line_names(self);
	if (!ret)
		return FALSE;

	return g_gpiosim_chip_apply_hogs(self);
}

static gboolean g_gpiosim_chip_enable(GPIOSimChip *self)
{
	struct gpiosim_dev *dev;
	int ret;

	dev = gpiosim_bank_get_dev(self->bank);
	ret = gpiosim_dev_enable(dev);
	gpiosim_dev_unref(dev);
	if (ret) {
		g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_CHIP_ENABLE_FAILED,
			    "Error while trying to enable the simulated GPIO device: %s",
			    g_strerror(errno));
		return FALSE;
	}

	return TRUE;
}

static void g_gpiosim_chip_constructed(GObject *obj)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP(obj);
	gboolean ret;

	ret = g_gpiosim_chip_apply_properties(self);
	if (!ret)
		return;

	ret = g_gpiosim_chip_enable(self);
	if (!ret)
		return;

	G_OBJECT_CLASS(g_gpiosim_chip_parent_class)->constructed(obj);
}

static void g_gpiosim_chip_get_property(GObject *obj, guint prop_id,
					GValue *val, GParamSpec *pspec)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP(obj);

	switch (prop_id) {
	case G_GPIOSIM_CHIP_PROP_DEV_PATH:
		g_value_set_static_string(val,
				gpiosim_bank_get_dev_path(self->bank));
		break;
	case G_GPIOSIM_CHIP_PROP_NAME:
		g_value_set_static_string(val,
				gpiosim_bank_get_chip_name(self->bank));
		break;
	default:
		G_OBJECT_WARN_INVALID_PROPERTY_ID(obj, prop_id, pspec);
		break;
	}
}

static void set_variant_prop(GVariant **prop, const GValue *val)
{
	GVariant *variant = g_value_get_variant(val);

	g_clear_pointer(prop, g_variant_unref);
	if (variant)
		*prop = g_variant_ref(variant);
}

static void g_gpiosim_chip_set_property(GObject *obj, guint prop_id,
					const GValue *val, GParamSpec *pspec)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP(obj);
	const gchar *label;

	switch (prop_id) {
	case G_GPIOSIM_CHIP_PROP_NUM_LINES:
		self->num_lines = g_value_get_uint(val);
		break;
	case G_GPIOSIM_CHIP_PROP_LABEL:
		g_clear_pointer(&self->label, g_free);
		label = g_value_get_string(val);
		if (label)
			self->label = g_strdup(label);
		break;
	case G_GPIOSIM_CHIP_PROP_LINE_NAMES:
		set_variant_prop(&self->line_names, val);
		break;
	case G_GPIOSIM_CHIP_PROP_HOGS:
		set_variant_prop(&self->hogs, val);
		break;
	default:
		G_OBJECT_WARN_INVALID_PROPERTY_ID(obj, prop_id, pspec);
		break;
	}
}

static void g_gpiosim_chip_dispose(GObject *obj)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP(obj);

	g_clear_pointer(&self->line_names, g_variant_unref);
	g_clear_pointer(&self->hogs, g_variant_unref);

	G_OBJECT_CLASS(g_gpiosim_chip_parent_class)->dispose(obj);
}

static void g_gpiosim_chip_finalize(GObject *obj)
{
	GPIOSimChip *self = G_GPIOSIM_CHIP(obj);
	struct gpiosim_dev *dev;
	gint ret;

	g_clear_error(&self->construct_err);
	g_clear_pointer(&self->label, g_free);

	dev = gpiosim_bank_get_dev(self->bank);

	if (gpiosim_dev_is_live(dev)) {
		ret = gpiosim_dev_disable(dev);
		if (ret)
			g_warning("Error while trying to disable the simulated GPIO device: %s",
				  g_strerror(errno));
	}

	gpiosim_dev_unref(dev);
	gpiosim_bank_unref(self->bank);

	G_OBJECT_CLASS(g_gpiosim_chip_parent_class)->finalize(obj);
}

static void g_gpiosim_chip_class_init(GPIOSimChipClass *chip_class)
{
	GObjectClass *class = G_OBJECT_CLASS(chip_class);

	class->constructed = g_gpiosim_chip_constructed;
	class->get_property = g_gpiosim_chip_get_property;
	class->set_property = g_gpiosim_chip_set_property;
	class->dispose = g_gpiosim_chip_dispose;
	class->finalize = g_gpiosim_chip_finalize;

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_DEV_PATH,
		g_param_spec_string("dev-path", "Device path",
				    "Character device filesystem path.", NULL,
				    G_PARAM_READABLE));

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_NAME,
		g_param_spec_string(
			"name", "Chip name",
			"Name of this chip device as set by the kernel.", NULL,
			G_PARAM_READABLE));

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_NUM_LINES,
		g_param_spec_uint("num-lines", "Number of lines",
				  "Number of lines this simulated chip exposes.",
				  1, G_MAXUINT, 1,
				  G_PARAM_WRITABLE | G_PARAM_CONSTRUCT_ONLY));

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_LABEL,
		g_param_spec_string("label", "Chip label",
				    "Label of this simulated chip.", NULL,
				    G_PARAM_WRITABLE | G_PARAM_CONSTRUCT_ONLY));

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_LINE_NAMES,
		g_param_spec_variant(
			"line-names", "Line names",
			"List of names of the lines exposed by this chip",
			(GVariantType *)"a(us)", NULL,
			G_PARAM_WRITABLE | G_PARAM_CONSTRUCT_ONLY));

	g_object_class_install_property(
		class, G_GPIOSIM_CHIP_PROP_HOGS,
		g_param_spec_variant(
			"hogs", "Line hogs",
			"List of hogged lines and their directions.",
			(GVariantType *)"a(usi)", NULL,
			G_PARAM_WRITABLE | G_PARAM_CONSTRUCT_ONLY));
}

static gboolean g_gpiosim_ctx_init(GError **err)
{
	sim_ctx = gpiosim_ctx_new();
	if (!sim_ctx) {
		g_set_error(err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_CTX_INIT_FAILED,
			    "Unable to initialize libgpiosim: %s",
			    g_strerror(errno));
		return FALSE;
	}

	atexit(g_gpiosim_ctx_unref);

	return TRUE;
}

static void g_gpiosim_chip_init(GPIOSimChip *self)
{
	struct gpiosim_dev *dev;
	int ret;

	self->construct_err = NULL;
	self->num_lines = 1;
	self->label = NULL;
	self->line_names = NULL;
	self->hogs = NULL;

	if (!sim_ctx) {
		ret = g_gpiosim_ctx_init(&self->construct_err);
		if (!ret)
			return;
	}

	dev = gpiosim_dev_new(sim_ctx);
	if (!dev) {
		g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
			    "Unable to instantiate new GPIO device: %s",
			    g_strerror(errno));
		return;
	}

	self->bank = gpiosim_bank_new(dev);
	gpiosim_dev_unref(dev);
	if (!self->bank) {
		g_set_error(&self->construct_err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_CHIP_INIT_FAILED,
			    "Unable to instantiate new GPIO bank: %s",
			    g_strerror(errno));
		return;
	}
}

static const gchar *
g_gpiosim_chip_get_string_prop(GPIOSimChip *self, const gchar *prop)
{
	GValue val = G_VALUE_INIT;
	const gchar *str;

	g_object_get_property(G_OBJECT(self), prop, &val);
	str = g_value_get_string(&val);
	g_value_unset(&val);

	return str;
}

const gchar *g_gpiosim_chip_get_dev_path(GPIOSimChip *self)
{
	return g_gpiosim_chip_get_string_prop(self, "dev-path");
}

const gchar *g_gpiosim_chip_get_name(GPIOSimChip *self)
{
	return g_gpiosim_chip_get_string_prop(self, "name");
}

gint _g_gpiosim_chip_get_value(GPIOSimChip *chip, guint offset, GError **err)
{
	gint val;

	val = gpiosim_bank_get_value(chip->bank, offset);
	if (val < 0) {
		g_set_error(err, G_GPIOSIM_ERROR,
			    G_GPIOSIM_ERR_GET_VALUE_FAILED,
			    "Unable to read the line value: %s",
			    g_strerror(errno));
		return -1;
	}

	return val;
}

void g_gpiosim_chip_set_pull(GPIOSimChip *chip, guint offset, GPIOSimPull pull)
{
	gint ret, sim_pull;

	switch (pull) {
	case G_GPIOSIM_PULL_DOWN:
		sim_pull = GPIOSIM_PULL_DOWN;
		break;
	case G_GPIOSIM_PULL_UP:
		sim_pull = GPIOSIM_PULL_UP;
		break;
	default:
		g_error("invalid pull value");
	}

	ret = gpiosim_bank_set_pull(chip->bank, offset, sim_pull);
	if (ret)
		g_critical("Unable to set the pull setting for simulated line: %s",
			    g_strerror(errno));
}
