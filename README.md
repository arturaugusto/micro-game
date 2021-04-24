# Micro-game

![alt tag](img.jpg)

## Learning Rust, embedded devices (STM32), game development and having fun.

Gameplay 1: https://youtu.be/5nPGiNsgY70
Gameplay 2: https://youtu.be/BnHSmtdbuVA
Gameplay 3: https://youtu.be/5l7y0dTvIQc


## watch and build:
cargo watch -cx 'build --release'

## watch + upload :
cargo watch -cx 'flash --chip stm32f103C8 --release'


## Wireup


 Tables        | Are           | Cool  
 ------------- |:-------------:| -----:
 col 3 is      | right-aligned | $1600 
 col 2 is      | centered      |   $12 
 zebra stripes | are neat      |    $1 




 Display | STM32
 -- | --
GND | G 
VCC | 3.3  
D0  | A5  
D1  | A7 
RES | B0 
DC  | B10 
CS  | G 



Analog button | STM32
-- | --
GND | G
+5  | 3.3
VRx | A1
VRy | A2
SW  | 


Digital button | STM32
-- | --
VCC | 3.3 
OUT | B1 
GND | G 



Buzzer

B9 R200 +Buzzer- G

Bluetooth | STM32
-- | --
STATE | 
RXD   | B6
TXD   | B7
GND   | G
VCC   | 3.3
EN    | 

