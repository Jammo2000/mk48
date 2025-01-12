// SPDX-FileCopyrightText: 2022 Softbear, Inc.

use yew::prelude::*;

#[function_component(BorkFlag)]
pub fn bork_flag() -> Html {
    html! {
        <svg
            version="1.1"
            id="bork_svg"
            style="width: 100%;"
            viewBox="0 0 50 30"
            xmlns="http://www.w3.org/2000/svg"
            >
            <g transform="matrix(0.48 0 0 0.29 25.22 14.44)">
                <polygon style="stroke: rgb(0,0,0); stroke-width: 5; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(0,0,0); fill-rule: nonzero; opacity: 1;"  points="-50,-50 -50,50 50,50 50,-50 " />
            </g>
            <g transform="matrix(0.28 0 0 0.28 25.25 14.58)">
                <circle style="stroke: rgb(255,255,255); stroke-width: 8; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(255,255,255); fill-rule: nonzero; opacity: 1;"  cx="0" cy="0" r="40" />
            </g>
            <g transform="matrix(0.06 0 0 0.06 23.33 11.33)">
                <circle style="stroke: rgb(0,0,0); stroke-width: 8; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(0,0,0); fill-rule: nonzero; opacity: 1;"  cx="0" cy="0" r="40" />
            </g>
                <g transform="matrix(0.06 0 0 0.06 30.64 11.64)">
            <circle style="stroke: rgb(0,0,0); stroke-width: 8; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(0,0,0); fill-rule: nonzero; opacity: 1;"  cx="0" cy="0" r="40" />
            </g>
            <g transform="matrix(0.11 0 0 0.06 27.64 17.86)">
                <circle style="stroke: rgb(0,0,0); stroke-width: 8; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(0,0,0); fill-rule: nonzero; opacity: 1;"  cx="0" cy="0" r="40" />
            </g>
            <g transform="matrix(0.06 0 0 0.06 27.64 19.64)">
                <circle style="stroke: rgb(0,0,0); stroke-width: 8; stroke-dasharray: none; stroke-linecap: butt; stroke-dashoffset: 0; stroke-linejoin: miter; stroke-miterlimit: 4; fill: rgb(0,0,0); fill-rule: nonzero; opacity: 1;"  cx="0" cy="0" r="40" />
            </g>
        </svg>
    }
}
