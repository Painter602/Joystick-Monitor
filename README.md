# <img src="assets/JSIcon.png"> Joystick Monitor
## Purpose
A program to echo joystick actions in a window.  The window has a green background and is intended to be monitored as a wondow in OBS studio.
Note: this is not written as a plugin for OBS, but could be adapted.
## Images
### Sticks
Joystick images are .svg files, 240 pixels square.  Other sizes should work, but have not been tested.

<img src="/img/00.svg" width="240" /> 00.svg Strafe stick  
<img src="/img/01.svg"  width="240" /> 01.svg Pitch & Roll stick  
<img src="/img/99.svg"  width="240" /> 99.svg Stick centre  
### Buttons
Buttons are square .svg images, not more than 1/8 the size of the Joystick images, preferably smaller.

1/8 - 12 would works well; i.e. 240/8-12 -> 68 pixels (widget marines default to 6 pixels either side on my system).

<img src="/img/b_10.svg" width="30" /> b_10.svg Button 1  
<img src="/img/b_13.svg" width="30" /> b_10.svg Other buttons, on  
<img src="/img/b_99.svg" width="30" /> b_10.svg Buttons, off

## To Do
- Adapt to work with HOTAS set-ups (Hands On Throttle And Stick). Games throttles often have two, side-by-side, sliders.
- Display other stick/slider inputs
- Simplify finding images in memory; the program's uses a quirky approach to storing and finding images in memory.
