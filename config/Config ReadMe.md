# Config read me
Configuration is in a file called joystick_monitor.ini

For now, it needs to be adapted by hand for your set-up.

## File layout
The file conforms to common .ini file designs.

It consists of a number of sections.  Section names appear between square brackets. Each section has a collection of key=value pairs.
```
[Right JS]
   vid = 3344
   pid = C0CC
   comment = other allowed key=value pairs....
  
[Comment]
This is a comment section.  This, together with any associated key=value pairs it has, will be ignored
   a_key = a_value
   key_name = some_value
   comment = this is a value for a comment, it will be ignored 
```
## Keys
### Required keys
Each device (joystick, throttle, udder pedal, or ?other?)
- vid: a four digit hex code, used to identify the device's vendor (maker)
- pid: a four digit hex code, used to identift the device's product code
The vid and pid, together, need to form a unique value.  This program will not work with two identical, non-configurable, joysticks.  
Throttles generally have different pid's to their associated joysticks.
Joysticks designed to work together will, usually, have a means to configure the pid (either a switch, or some configuration software).
### Optional keys
- x = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- y = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- z = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- col = col (number)
- log_device = true/false (default)
- buttons = comma separated list of numbers
- echo_x = hex-code hex-code
- echo_y = hex-code hex-code
- echo_z = hex-code hex-code
- comment = use to describe your intent, ignored by the program.
### Keys, allowed but not yet implemented
These keys may be stored in the configuration file, data will be collected, but not used
- rx = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- ry = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- rz = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- slider_0 = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
- slider_1 = axis (eg: n0 (number), n1 (number), label (text), invert (true/false), calibrate (number))
### Unknown keys
In joystick sections, unknown keys will be logged in the current session's log file.

### Axis fields
Axies consist of up-to five, comma separated, fields
- Field 1: reading, offset to small value in the buffer
- Field 2: reading, offset to high value in the buffer
- Field 3: label, text.  **Must not contain a comma**
- Field 4: invert, reverse the axis reading
- Field 5: calibrate, adjust the neutal centre of the device in display.

## To do
Revise the way axis fields are set-up.
Improve this document
