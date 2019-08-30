#include <miral/window_info.h>
#include <miral/window_manager_tools.h>

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

extern "C" void select_active_window(miral::WindowManagerTools* tools, miral::Window const* hint)
{
    printf("tools %p, hint: %p\n", (void*) tools, (void*) hint);
    tools->select_active_window(*hint);
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