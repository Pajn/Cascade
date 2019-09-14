#include <mir/scene/surface.h>
#include <miral/wayland_extensions.h>
#include <miral/window_info.h>
#include <miral/window_manager_tools.h>
#include <wayland-server-core.h>

extern "C" auto window_specification_name(miral::WindowSpecification& specification) -> const char*
{
    if (specification.name().is_set()) {
        return specification.name().value().c_str();
    } else {
        return nullptr;
    }
}

extern "C" auto configure_window(miral::WindowInfo& window_info, MirWindowAttrib attrib, int value) -> int
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    return surface.get()->configure(attrib, value);
}

extern "C" auto hide_window(miral::WindowInfo& window_info) -> void
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    surface.get()->hide();
}

extern "C" auto show_window(miral::WindowInfo& window_info) -> void
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    surface.get()->show();
}

extern "C" auto window_name(miral::WindowInfo& window_info) -> void*
{
    auto name = window_info.name();
    auto name_ptr = std::make_shared<std::string>();
    *name_ptr = name;
    return static_cast<void*>(new std::shared_ptr<std::string>(name_ptr));
}

extern "C" auto rust_get_string(std::shared_ptr<std::string> string) -> const char*
{
    return string.get()->c_str();
}

extern "C" auto rust_drop_string(void* ptr) -> void
{
    std::shared_ptr<std::string> *value = static_cast<std::shared_ptr<std::string>*>(ptr);
    delete value;
}

extern "C" auto window_specification_has_parent(miral::WindowSpecification& specification) -> bool
{
    return specification.parent().is_set() && specification.parent().value().lock();
}
extern "C" auto window_info_has_parent(miral::WindowInfo& window_info) -> bool
{
    return !!window_info.parent();
}

extern "C" auto get_active_window(miral::WindowManagerTools* tools) -> void*
{
    auto window = tools->active_window();
    auto window_ptr = std::make_shared<miral::Window>();
    *window_ptr = window;
    return static_cast<void*>(new std::shared_ptr<miral::Window>(window_ptr));
}

extern "C" auto get_window_at(miral::WindowManagerTools* tools, mir::geometry::Point cursor) -> void*
{
    auto window = tools->window_at(cursor);
    auto window_ptr = std::make_shared<miral::Window>();
    *window_ptr = window;
    return static_cast<void*>(new std::shared_ptr<miral::Window>(window_ptr));
}

extern "C" auto select_active_window(miral::WindowManagerTools* tools, miral::Window const* hint) -> void
{
    if (hint == NULL) {
        tools->select_active_window(miral::Window {});
    } else {
        tools->select_active_window(*hint);
    }
}

extern "C" auto rust_get_window(std::shared_ptr<miral::Window> window) -> miral::Window*
{
    return window.get();
}

extern "C" auto rust_drop_window(void* ptr) -> void
{
    std::shared_ptr<miral::Window> *value = static_cast<std::shared_ptr<miral::Window>*>(ptr);
    delete value;
}

extern "C" auto client_is_alive(wl_client* client) -> bool
{
    return miral::application_for(client) != NULL;
}

extern "C" auto client_owns_window(wl_client* client, miral::WindowInfo const* window) -> bool
{
    return miral::pid_of(miral::application_for(client)) == miral::pid_of(window->window().application());
}