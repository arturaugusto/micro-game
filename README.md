# Micro-game

![alt tag](img.jpg)

## Learning Rust, embedded devices (stm32), game development and having fun.

Gameplay 1: https://youtu.be/5nPGiNsgY70
Gameplay 2: https://youtu.be/BnHSmtdbuVA
Gameplay 3: https://youtu.be/5l7y0dTvIQc


watch and build:
cargo watch -cx 'build --release'

watch + upload :
cargo watch -cx 'flash --chip stm32f103C8 --release'


## Connections


Display
GND | G
VCC | 3.3
D0  | A5
D1  | A7
RES | B0
DC  | B10
CS  | G

Analog button
GND | G
+5  | 3.3
VRx | A1
VRy | A2
SW  | 

Digital button
VCC | 3.3
OUT | B1
GND | G

Buzzer

B9 R200 +Buzzer- G

Bluetooth

STATE | 
RXD   | B6
TXD   | B7
GND   | G
VCC   | 3.3
EN    | 