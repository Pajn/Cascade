#include <mir/scene/surface.h>
#include <miral/wayland_extensions.h>
#include <miral/window_info.h>
#include <miral/window_manager_tools.h>
#include <wayland-server-core.h>

extern "C" const char* window_specification_name(miral::WindowSpecification& specification)
{
    if (specification.name().is_set()) {
        return specification.name().value().c_str();
    } else {
        return nullptr;
    }
}

extern "C" int configure_window(miral::WindowInfo& window_info, MirWindowAttrib attrib, int value)
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    return surface.get()->configure(attrib, value);
}

extern "C" void hide_window(miral::WindowInfo& window_info)
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    surface.get()->hide();
}

extern "C" void show_window(miral::WindowInfo& window_info)
{
    std::shared_ptr<mir::scene::Surface> surface = window_info.window();
    surface.get()->show();
}

extern "C" void* window_name(miral::WindowInfo& window_info)
{
    auto name = window_info.name();
    auto name_ptr = std::make_shared<std::string>();
    *name_ptr = name;
    return static_cast<void*>(new std::shared_ptr<std::string>(name_ptr));
}

extern "C" const char* rust_get_string(std::shared_ptr<std::string> string)
{
    return string.get()->c_str();
}

extern "C" void rust_drop_string(void* ptr)
{
    std::shared_ptr<std::string> *value = static_cast<std::shared_ptr<std::string>*>(ptr);
    delete value;
}

extern "C" bool window_specification_has_parent(miral::WindowSpecification& specification)
{
    return specification.parent().is_set() && specification.parent().value().lock();
}
extern "C" bool window_info_has_parent(miral::WindowInfo& window_info)
{
    return !!window_info.parent();
}

extern "C" void* get_active_window(miral::WindowManagerTools* tools)
{
    auto window = tools->active_window();
    auto window_ptr = std::make_shared<miral::Window>();
    *window_ptr = window;
    return static_cast<void*>(new std::shared_ptr<miral::Window>(window_ptr));
}

extern "C" void* get_window_at(miral::WindowManagerTools* tools, mir::geometry::Point cursor)
{
    auto window = tools->window_at(cursor);
    auto window_ptr = std::make_shared<miral::Window>();
    *window_ptr = window;
    return static_cast<void*>(new std::shared_ptr<miral::Window>(window_ptr));
}

extern "C" void select_active_window(miral::WindowManagerTools* tools, miral::Window const* hint)
{
    if (hint == NULL) {
        tools->select_active_window(miral::Window {});
    } else {
        tools->select_active_window(*hint);
    }
}

extern "C" miral::Window* rust_get_window(std::shared_ptr<miral::Window> window)
{
    return window.get();
}

extern "C" void rust_drop_window(void* ptr)
{
    std::shared_ptr<miral::Window> *value = static_cast<std::shared_ptr<miral::Window>*>(ptr);
    delete value;
}

extern "C" bool client_is_alive(wl_client* client)
{
    return miral::application_for(client) != NULL;
}

extern "C" bool client_owns_window(wl_client* client, miral::WindowInfo const* window)
{
    return miral::pid_of(miral::application_for(client)) == miral::pid_of(window->window().application());
}