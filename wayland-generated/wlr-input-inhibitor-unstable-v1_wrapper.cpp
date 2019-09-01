/*
 * AUTOGENERATED - DO NOT EDIT
 *
 * This file is generated from wlr-input-inhibitor-unstable-v1.xml
 * To regenerate, run the “refresh-wayland-wrapper” target.
 */

#include "wlr-input-inhibitor-unstable-v1_wrapper.h"

#include <boost/throw_exception.hpp>
#include <boost/exception/diagnostic_information.hpp>

#include <wayland-server-core.h>

#include "mir/log.h"

namespace mir
{
namespace wayland
{
extern struct wl_interface const zwlr_input_inhibit_manager_v1_interface_data;
extern struct wl_interface const zwlr_input_inhibitor_v1_interface_data;
}
}

namespace mw = mir::wayland;

namespace
{
struct wl_interface const* all_null_types [] {
    nullptr,
    nullptr,
    nullptr,
    nullptr,
    nullptr,
    nullptr};
}

// ZwlrInputInhibitManagerV1

mw::ZwlrInputInhibitManagerV1* mw::ZwlrInputInhibitManagerV1::from(struct wl_resource* resource)
{
    return static_cast<ZwlrInputInhibitManagerV1*>(wl_resource_get_user_data(resource));
}

struct mw::ZwlrInputInhibitManagerV1::Thunks
{
    static int const supported_version;

    static void get_inhibitor_thunk(struct wl_client* client, struct wl_resource* resource, uint32_t id)
    {
        auto me = static_cast<ZwlrInputInhibitManagerV1*>(wl_resource_get_user_data(resource));
        wl_resource* id_resolved{
            wl_resource_create(client, &zwlr_input_inhibitor_v1_interface_data, wl_resource_get_version(resource), id)};
        if (id_resolved == nullptr)
        {
            wl_client_post_no_memory(client);
            BOOST_THROW_EXCEPTION((std::bad_alloc{}));
        }
        try
        {
            me->get_inhibitor(id_resolved);
        }
        catch(...)
        {
            internal_error_processing_request(client, "ZwlrInputInhibitManagerV1::get_inhibitor()");
        }
    }

    static void resource_destroyed_thunk(wl_resource* resource)
    {
        delete static_cast<ZwlrInputInhibitManagerV1*>(wl_resource_get_user_data(resource));
    }

    static void bind_thunk(struct wl_client* client, void* data, uint32_t version, uint32_t id)
    {
        auto me = static_cast<ZwlrInputInhibitManagerV1::Global*>(data);
        auto resource = wl_resource_create(
            client,
            &zwlr_input_inhibit_manager_v1_interface_data,
            std::min((int)version, Thunks::supported_version),
            id);
        if (resource == nullptr)
        {
            wl_client_post_no_memory(client);
            BOOST_THROW_EXCEPTION((std::bad_alloc{}));
        }
        try
        {
            me->bind(resource);
        }
        catch(...)
        {
            internal_error_processing_request(client, "ZwlrInputInhibitManagerV1 global bind");
        }
    }

    static struct wl_interface const* get_inhibitor_types[];
    static struct wl_message const request_messages[];
    static void const* request_vtable[];
};

int const mw::ZwlrInputInhibitManagerV1::Thunks::supported_version = 1;

mw::ZwlrInputInhibitManagerV1::ZwlrInputInhibitManagerV1(struct wl_resource* resource, Version<1>)
    : client{wl_resource_get_client(resource)},
      resource{resource}
{
    if (resource == nullptr)
    {
        BOOST_THROW_EXCEPTION((std::bad_alloc{}));
    }
    wl_resource_set_implementation(resource, Thunks::request_vtable, this, &Thunks::resource_destroyed_thunk);
}

bool mw::ZwlrInputInhibitManagerV1::is_instance(wl_resource* resource)
{
    return wl_resource_instance_of(resource, &zwlr_input_inhibit_manager_v1_interface_data, Thunks::request_vtable);
}

void mw::ZwlrInputInhibitManagerV1::destroy_wayland_object() const
{
    wl_resource_destroy(resource);
}

mw::ZwlrInputInhibitManagerV1::Global::Global(wl_display* display, Version<1>)
    : wayland::Global{
          wl_global_create(
              display,
              &zwlr_input_inhibit_manager_v1_interface_data,
              Thunks::supported_version,
              this,
              &Thunks::bind_thunk)}
{}

auto mw::ZwlrInputInhibitManagerV1::Global::interface_name() const -> char const*
{
    return ZwlrInputInhibitManagerV1::interface_name;
}

struct wl_interface const* mw::ZwlrInputInhibitManagerV1::Thunks::get_inhibitor_types[] {
    &zwlr_input_inhibitor_v1_interface_data};

struct wl_message const mw::ZwlrInputInhibitManagerV1::Thunks::request_messages[] {
    {"get_inhibitor", "n", get_inhibitor_types}};

void const* mw::ZwlrInputInhibitManagerV1::Thunks::request_vtable[] {
    (void*)Thunks::get_inhibitor_thunk};

// ZwlrInputInhibitorV1

mw::ZwlrInputInhibitorV1* mw::ZwlrInputInhibitorV1::from(struct wl_resource* resource)
{
    return static_cast<ZwlrInputInhibitorV1*>(wl_resource_get_user_data(resource));
}

struct mw::ZwlrInputInhibitorV1::Thunks
{
    static int const supported_version;

    static void destroy_thunk(struct wl_client* client, struct wl_resource* resource)
    {
        auto me = static_cast<ZwlrInputInhibitorV1*>(wl_resource_get_user_data(resource));
        try
        {
            me->destroy();
        }
        catch(...)
        {
            internal_error_processing_request(client, "ZwlrInputInhibitorV1::destroy()");
        }
    }

    static void resource_destroyed_thunk(wl_resource* resource)
    {
        delete static_cast<ZwlrInputInhibitorV1*>(wl_resource_get_user_data(resource));
    }

    static struct wl_message const request_messages[];
    static void const* request_vtable[];
};

int const mw::ZwlrInputInhibitorV1::Thunks::supported_version = 1;

mw::ZwlrInputInhibitorV1::ZwlrInputInhibitorV1(struct wl_resource* resource, Version<1>)
    : client{wl_resource_get_client(resource)},
      resource{resource}
{
    if (resource == nullptr)
    {
        BOOST_THROW_EXCEPTION((std::bad_alloc{}));
    }
    wl_resource_set_implementation(resource, Thunks::request_vtable, this, &Thunks::resource_destroyed_thunk);
}

bool mw::ZwlrInputInhibitorV1::is_instance(wl_resource* resource)
{
    return wl_resource_instance_of(resource, &zwlr_input_inhibitor_v1_interface_data, Thunks::request_vtable);
}

void mw::ZwlrInputInhibitorV1::destroy_wayland_object() const
{
    wl_resource_destroy(resource);
}

struct wl_message const mw::ZwlrInputInhibitorV1::Thunks::request_messages[] {
    {"destroy", "", all_null_types}};

void const* mw::ZwlrInputInhibitorV1::Thunks::request_vtable[] {
    (void*)Thunks::destroy_thunk};

namespace mir
{
namespace wayland
{

struct wl_interface const zwlr_input_inhibit_manager_v1_interface_data {
    mw::ZwlrInputInhibitManagerV1::interface_name,
    mw::ZwlrInputInhibitManagerV1::Thunks::supported_version,
    1, mw::ZwlrInputInhibitManagerV1::Thunks::request_messages,
    0, nullptr};

struct wl_interface const zwlr_input_inhibitor_v1_interface_data {
    mw::ZwlrInputInhibitorV1::interface_name,
    mw::ZwlrInputInhibitorV1::Thunks::supported_version,
    1, mw::ZwlrInputInhibitorV1::Thunks::request_messages,
    0, nullptr};

}
}