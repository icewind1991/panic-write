# panic-write

Write panic messages to a `core::fmt::Write` and then halt, intended for bare metal development.

## Usage

```
#![no_std]

use panic_write::PanicHandler;
use core::fmt::Write;

let serial = ...;
// assign the handler to an unused variable to stop it from getting dropped
let _panic_handler = PanicHandler::new(serial);
```

The panic handler is un-registered when dropped, if no active panic handler is registered and the app panics, it will halt without printing anything.

Additionally, the panic handler can also be used in place of the original `Write` throughout the rest of the app.

```
#![no_std]

use panic_write::PanicHandler;
use core::fmt::Write;

let serial = ...;
let mut serial = PanicHandler::new(serial);

writeln!(&mut serial, "starting app");
```
