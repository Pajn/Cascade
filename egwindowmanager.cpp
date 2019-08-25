/*
 * Copyright © 2016-19 Octopull Ltd.
 *
 * This program is free software: you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 3,
 * as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Authored by: Alan Griffiths <alan@octopull.co.uk>
 */

#include "rust_wm/rust_wm.h"
#include "egwindowmanager.h"
#include "egwallpaper.h"
#include "eglauncher.h"

#include <miral/application_info.h>
#include <miral/window_info.h>
#include <miral/window_manager_tools.h>

#include <linux/input.h>

using namespace mir::geometry;


void egmde::WindowManagerPolicy::keep_size_within_limits(
    WindowInfo const& window_info, Displacement& delta, Width& new_width, Height& new_height) const
{
    auto const min_width  = std::max(window_info.min_width(), Width{5});
    auto const min_height = std::max(window_info.min_height(), Height{5});

    if (new_width < min_width)
    {
        new_width = min_width;
        if (delta.dx > DeltaX{0})
            delta.dx = DeltaX{0};
    }

    if (new_height < min_height)
    {
        new_height = min_height;
        if (delta.dy > DeltaY{0})
            delta.dy = DeltaY{0};
    }

    auto const max_width  = window_info.max_width();
    auto const max_height = window_info.max_height();

    if (new_width > max_width)
    {
        new_width = max_width;
        if (delta.dx < DeltaX{0})
            delta.dx = DeltaX{0};
    }

    if (new_height > max_height)
    {
        new_height = max_height;
        if (delta.dy < DeltaY{0})
            delta.dy = DeltaY{0};
    }
}

egmde::WindowManagerPolicy::WindowManagerPolicy(WindowManagerTools const& tools, Wallpaper const& wallpaper, Launcher const& launcher) :
    MinimalWindowManager{tools},
    wallpaper{&wallpaper},
    launcher{&launcher}
{
    this->tools = std::make_shared<WindowManagerTools>(tools);
    wm = rust::init_wm(this->tools.get());
}

miral::WindowSpecification egmde::WindowManagerPolicy::place_new_window(
    miral::ApplicationInfo const& app_info, miral::WindowSpecification const& request_parameters)
{
    auto result = MinimalWindowManager::place_new_window(app_info, request_parameters);
    rust::place_new_window(wm, &result);

    if (app_info.application() == wallpaper->session())
    {
        result.type() = mir_window_type_decoration;
    }
    if (app_info.application() == launcher->session())
    {
        result.type() = mir_window_type_dialog;
    }

    return result;
}

bool egmde::WindowManagerPolicy::handle_keyboard_event(MirKeyboardEvent const* event)
{
    return rust::handle_keyboard_event(wm, event) 
        || MinimalWindowManager::handle_keyboard_event(event);
}

void egmde::WindowManagerPolicy::handle_window_ready(WindowInfo& window_info)
{
    MinimalWindowManager::handle_window_ready(window_info);
    rust::handle_window_ready(wm, &window_info);
}

void egmde::WindowManagerPolicy::advise_focus_gained(WindowInfo const& window_info)
{
    MinimalWindowManager::advise_focus_gained(window_info);
    rust::advise_focus_gained(wm, &window_info);
}
void egmde::WindowManagerPolicy::advise_delete_window(WindowInfo const& window_info)
{
    rust::advise_delete_window(wm, &window_info);
}
void egmde::WindowManagerPolicy::handle_modify_window(WindowInfo& window_info, WindowSpecification const& modifications)
{
    auto specification = modifications;
    rust::pre_handle_modify_window(wm, &window_info, &specification);
    MinimalWindowManager::handle_modify_window(window_info, specification);
    rust::post_handle_modify_window(wm, &window_info, &specification);
}

void egmde::WindowManagerPolicy::advise_application_zone_create(miral::Zone const& zone)
{
    rust::advise_application_zone_create(wm, &zone);
}
void egmde::WindowManagerPolicy::advise_application_zone_update(miral::Zone const& updated, miral::Zone const& original)
{
    rust::advise_application_zone_update(wm, &updated, &original);
}
void egmde::WindowManagerPolicy::advise_application_zone_delete(miral::Zone const& zone)
{
    rust::advise_application_zone_delete(wm, &zone);
}

void egmde::WindowManagerPolicy::advise_output_create(Output const& output)
{
    rust::advise_output_create(wm, &output);
}
void egmde::WindowManagerPolicy::advise_output_update(Output const& updated, Output const& original)
{
    rust::advise_output_update(wm, &updated, &original);
}
void egmde::WindowManagerPolicy::advise_output_delete(Output const& output)
{
    rust::advise_output_delete(wm, &output);
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
    Window window = tools->active_window();
    std::shared_ptr<Window> window_ptr = std::make_shared<Window>();
    *window_ptr = window;
    return static_cast<void*>(new std::shared_ptr<Window>(window_ptr));
}

extern "C" void select_active_window(miral::WindowManagerTools* tools, Window const* hint)
{
    printf("tools %p, hint: %p\n", (void*) tools, (void*) hint);
    tools->select_active_window(*hint);
}

extern "C" Window* rust_get_window(std::shared_ptr<Window> window)
{
    return window.get();
}

extern "C" void rust_drop_window(void* ptr)
{
    std::shared_ptr<Window> *value = static_cast<std::shared_ptr<Window>*>(ptr);
    delete value;
}