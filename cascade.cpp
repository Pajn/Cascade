/*
 * Copyright © 2016-2019 Octopull Ltd.
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

#include "egwallpaper.h"
#include "egwindowmanager.h"
#include "eglauncher.h"
#include "wlr_input_inhibitor.h"
#include "rust_wm/rust_wm.h"

#include <miral/append_event_filter.h>
#include <miral/command_line_option.h>
#include <miral/display_configuration_option.h>
#include <miral/internal_client.h>
#include <miral/keymap.h>
#include <miral/runner.h>
#include <miral/set_window_management_policy.h>
#include <miral/wayland_extensions.h>

#include <miral/x11_support.h>
#include <miral/display_configuration.h>

#include <linux/input.h>

using namespace miral;

int main(int argc, char const* argv[])
{
    MirRunner runner{argc, argv};

    auto keymap = Keymap{};
    auto ipc_server = rust::init_ipc_server();
    auto input_inhibitor = rust::input_inhibitor_new();
    cascade::Wallpaper wallpaper;

    ExternalClientLauncher external_client_launcher;
    cascade::Launcher launcher{external_client_launcher};

    auto const keyboard_shortcuts = [&](MirEvent const* event)
        {
            if (mir_event_get_type(event) != mir_event_type_input)
                return false;

            MirInputEvent const* input_event = mir_event_get_input_event(event);
            if (mir_input_event_get_type(input_event) != mir_input_event_type_key)
                return false;

            MirKeyboardEvent const* kev = mir_input_event_get_keyboard_event(input_event);
            if (mir_keyboard_event_action(kev) != mir_keyboard_action_down)
                return false;

            MirInputEventModifiers mods = mir_keyboard_event_modifiers(kev);
            if (!(mods & mir_input_event_modifier_alt) || !(mods & mir_input_event_modifier_ctrl))
                return false;

            if (input_inhibitor_is_inhibited(input_inhibitor))
                return false;

            switch (mir_keyboard_event_scan_code(kev))
            {
            case KEY_A:launcher.show();
                return true;

            case KEY_BACKSPACE:
                runner.stop();
                return true;

            default:
                return false;
            }
        };


    runner.add_start_callback([&] { external_client_launcher.launch({"ulauncher", "--hide-window"}); });
    runner.add_stop_callback([&] { wallpaper.stop(); });
    runner.add_stop_callback([&] { launcher.stop(); });

    WaylandExtensions wayland_extensions;
    wayland_extensions.add_extension(cascade::wlr_input_inhibitor_extension(input_inhibitor));

    return runner.run_with(
        {
            X11Support{},
            wayland_extensions
                .enable(WaylandExtensions::zwlr_layer_shell_v1)
                .enable(WaylandExtensions::zxdg_output_manager_v1),
            DisplayConfiguration{runner},
            CommandLineOption{[&](auto& option) { wallpaper.top(option);},
                              "wallpaper-top",    "Colour of wallpaper RGB", "0x000000"},
            CommandLineOption{[&](auto& option) { wallpaper.bottom(option);},
                              "wallpaper-bottom", "Colour of wallpaper RGB", CASCADE_WALLPAPER_BOTTOM},
            StartupInternalClient{std::ref(wallpaper)},
            external_client_launcher,
            StartupInternalClient{std::ref(launcher)},
            keymap,
            AppendEventFilter{keyboard_shortcuts},
            set_window_management_policy<cascade::WindowManagerPolicy>(ipc_server, input_inhibitor, keymap, wallpaper, launcher)
        });
}
