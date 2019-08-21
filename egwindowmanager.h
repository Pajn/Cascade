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

#ifndef EGMDE_EGWINDOWMANAGER_H
#define EGMDE_EGWINDOWMANAGER_H

#include <miral/minimal_window_manager.h>
#include <miral/window_manager_tools.h>

namespace egmde
{
using namespace miral;
class Wallpaper;

class WindowManagerPolicy :
    public MinimalWindowManager
{
public:
    WindowManagerPolicy(WindowManagerTools const& tools, Wallpaper const& wallpaper);

    auto place_new_window(ApplicationInfo const& app_info, WindowSpecification const& request_parameters)
        -> WindowSpecification override;

    bool handle_keyboard_event(MirKeyboardEvent const* event) override;

private:
    miral::WindowManagerTools tools;
    Wallpaper const* wallpaper;

    void keep_size_within_limits(
        WindowInfo const& window_info, Displacement& delta, Width& new_width, Height& new_height) const;
};
}

#endif //EGMDE_EGWINDOWMANAGER_H
