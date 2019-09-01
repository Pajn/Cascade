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

#ifndef CASCADE_WLR_INPUT_INHIBITOR_H
#define CASCADE_WLR_INPUT_INHIBITOR_H

#include <miral/wayland_extensions.h>
#include "rust_wm/rust_wm.h"

namespace cascade
{
auto wlr_input_inhibitor_extension(rust::InputInhibitor* input_inhibitor) -> miral::WaylandExtensions::Builder;
}

#endif //CASCADE_WLR_INPUT_INHIBITOR_H