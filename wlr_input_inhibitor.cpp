/*
 * Copyright © 2019 Rasmus Eneman
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
 * Authored by: Rasmus Eneman <rasmus@eneman.eu>
 */

#include "wlr_input_inhibitor.h"
#include "rust_wm/rust_wm.h"

#include "wayland-generated/wlr-input-inhibitor-unstable-v1_wrapper.h"

using namespace mir::wayland;
using namespace miral;

namespace
{
class InputInhibitManager : public ZwlrInputInhibitManagerV1
{
public:
    InputInhibitManager(struct wl_resource* resource, rust::InputInhibitor* input_inhibitor);

private:
    rust::InputInhibitor* input_inhibitor;
    void get_inhibitor(struct wl_resource* id) override;
};

class InputInhibitor : public ZwlrInputInhibitorV1
{
public:
    InputInhibitor(
        struct wl_resource* resource, rust::InputInhibitor* input_inhibitor);

private:
    rust::InputInhibitor* input_inhibitor;
    void destroy() override;
};

class MyGlobal : public ZwlrInputInhibitManagerV1::Global
{
public:
    explicit MyGlobal(wl_display* display, rust::InputInhibitor* input_inhibitor);

private:
    rust::InputInhibitor* input_inhibitor;
    void bind(wl_resource* new_zwlr_input_inhibit_manager_v1) override;
};
}

void InputInhibitManager::get_inhibitor(struct wl_resource* id)
{
    rust::input_inhibitor_set(this->input_inhibitor, client);
    new InputInhibitor{id, this->input_inhibitor};
}

InputInhibitManager::InputInhibitManager(
    struct wl_resource* resource, rust::InputInhibitor* input_inhibitor) :
    ZwlrInputInhibitManagerV1(resource, Version<1>{}),
    input_inhibitor{input_inhibitor}
{
}

void InputInhibitor::destroy()
{
    rust::input_inhibitor_clear(this->input_inhibitor);
    destroy_wayland_object();
}

InputInhibitor::InputInhibitor(
    struct wl_resource* resource, rust::InputInhibitor* input_inhibitor) :
    ZwlrInputInhibitorV1(resource, Version<1>{}),
    input_inhibitor{input_inhibitor}
{
}

MyGlobal::MyGlobal(wl_display* display, rust::InputInhibitor* input_inhibitor __attribute__((unused))) :
    ZwlrInputInhibitManagerV1::Global(display, Version<1>{}),
    input_inhibitor{input_inhibitor}
{
}

void MyGlobal::bind(wl_resource* new_zwlr_input_inhibit_manager_v1)
{
    new InputInhibitManager{new_zwlr_input_inhibit_manager_v1, this->input_inhibitor};
}

auto cascade::wlr_input_inhibitor_extension(rust::InputInhibitor* input_inhibitor) -> WaylandExtensions::Builder
{
    return
        {
            InputInhibitManager::interface_name,
            [input_inhibitor](WaylandExtensions::Context const* context)
            {
                return std::make_shared<MyGlobal>(context->display(), input_inhibitor);
            }
        };
}