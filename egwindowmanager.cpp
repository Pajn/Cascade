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

#include <string>
#include <iostream>

#include <miral/application_info.h>
#include <miral/window_info.h>
#include <miral/wayland_extensions.h>
#include <miral/window_manager_tools.h>

#include <linux/input.h>

using namespace mir::geometry;


void cascade::WindowManagerPolicy::keep_size_within_limits(
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

cascade::WindowManagerPolicy::WindowManagerPolicy(WindowManagerTools const& tools, rust::InputInhibitor* input_inhibitor, Wallpaper const& wallpaper, Launcher const& launcher) :
    MinimalWindowManager{tools},
    input_inhibitor{input_inhibitor},
    wallpaper{&wallpaper},
    launcher{&launcher}
{
    this->tools = std::make_shared<WindowManagerTools>(tools);
    wm = rust::init_wm(this->tools.get(), this->input_inhibitor);
}

miral::WindowSpecification cascade::WindowManagerPolicy::place_new_window(
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

void cascade::WindowManagerPolicy::handle_window_ready(WindowInfo& window_info)
{

    MinimalWindowManager::handle_window_ready(window_info);
    rust::handle_window_ready(wm, &window_info);
}

void cascade::WindowManagerPolicy::handle_modify_window(WindowInfo& window_info, WindowSpecification const& modifications)
{
    auto specification = modifications;
    rust::pre_handle_modify_window(wm, &window_info, &specification);
    MinimalWindowManager::handle_modify_window(window_info, specification);
    rust::post_handle_modify_window(wm, &window_info, &specification);
}

void cascade::WindowManagerPolicy::advise_delete_window(WindowInfo const& window_info)
{
    rust::advise_delete_window(wm, &window_info);
}



bool cascade::WindowManagerPolicy::handle_keyboard_event(MirKeyboardEvent const* event)
{
    return rust::handle_keyboard_event(wm, event) 
        || MinimalWindowManager::handle_keyboard_event(event);
}
bool cascade::WindowManagerPolicy::handle_pointer_event(MirPointerEvent const* event)
{
    return rust::handle_pointer_event(wm, event);
}


void cascade::WindowManagerPolicy::handle_request_move(WindowInfo& window_info, MirInputEvent const* input_event)
{
    rust::handle_request_move(wm, &window_info, input_event);
}
void cascade::WindowManagerPolicy::handle_request_resize(WindowInfo& window_info, MirInputEvent const* input_event, MirResizeEdge edge)
{
    rust::handle_request_resize(wm, &window_info, input_event, edge);
}



void cascade::WindowManagerPolicy::handle_raise_window(WindowInfo& window_info)
{
    rust::handle_raise_window(wm, &window_info);
}
void cascade::WindowManagerPolicy::advise_focus_gained(WindowInfo const& window_info)
{
    MinimalWindowManager::advise_focus_gained(window_info);
    rust::advise_focus_gained(wm, &window_info);
}



void cascade::WindowManagerPolicy::advise_output_create(Output const& output)
{
    rust::advise_output_create(wm, &output);
}
void cascade::WindowManagerPolicy::advise_output_update(Output const& updated, Output const& original)
{
    rust::advise_output_update(wm, &updated, &original);
}
void cascade::WindowManagerPolicy::advise_output_delete(Output const& output)
{
    rust::advise_output_delete(wm, &output);
}



void cascade::WindowManagerPolicy::advise_application_zone_create(miral::Zone const& zone)
{
    rust::advise_application_zone_create(wm, &zone);
}
void cascade::WindowManagerPolicy::advise_application_zone_update(miral::Zone const& updated, miral::Zone const& original)
{
    rust::advise_application_zone_update(wm, &updated, &original);
}
void cascade::WindowManagerPolicy::advise_application_zone_delete(miral::Zone const& zone)
{
    rust::advise_application_zone_delete(wm, &zone);
}