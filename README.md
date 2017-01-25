# Droptest

A small crate to help test drop implementation

[![Build Status](https://travis-ci.org/malbarbo/testdrop.svg?branch=master)](https://travis-ci.org/malbarbo/testdrop)
[![Build status](https://ci.appveyor.com/api/projects/status/ww0qx6msilj8pwaw/branch/master?svg=true)](https://ci.appveyor.com/project/malbarbo/testdrop)
[![Crates](http://meritbadge.herokuapp.com/testdrop)](https://crates.io/crates/testdrop)

## Documentation

- [API documentation](https://docs.rs/testdrop)


### Example

Test if the [`std::rc::Rc`](https://doc.rust-lang.org/stable/std/rc/struct.Rc.html)
drop implementation works.

```rust
extern crate testdrop;

use testdrop::TestDrop;
use std::rc::Rc;

let td = TestDrop::new();
let (id, item) = td.new_item();
let item = Rc::new(item);
let item_clone = item.clone();

// Decrease the reference counter, but do not drop.
drop(item_clone);
td.assert_no_drop(id);

// Decrease the reference counter and then drop.
drop(item);
td.assert_drop(id);
```


## License

Licensed under either of

 - [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 - [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
