# `mod_rewrite`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/mod_rewrite?label=latest)](https://crates.io/crates/mod_rewrite)
[![Documentation](https://docs.rs/mod_rewrite/badge.svg?version=0.1.2)](https://docs.rs/mod_rewrite/0.1.2)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/mod_rewrite.svg)
<br />
[![dependency status](https://deps.rs/crate/mod_rewrite/0.1.2/status.svg)](https://deps.rs/crate/mod_rewrite/0.1.2)
[![Download](https://img.shields.io/crates/d/mod_rewrite.svg)](https://crates.io/crates/mod_rewrite)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Apache2 [`mod_rewrite`](https://httpd.apache.org/docs/current/mod/mod_rewrite.html)
reimplemented in rust for rust web-services.

## Examples

```rust
use mod_rewrite::Engine;

let engine = Engine::from_rules(r#"
  Rewrite /file/(.*)     /tmp/$1      [L]
  Rewrite /redirect/(.*) /location/$1 [R=302]
  Rewrite /blocked/(.*)  -            [F]
"#).expect("failed to process rules");

let uri = "http://localhost/file/my/document.txt".to_owned();
let result = engine.rewrite(uri).unwrap();
println!("{result:?}");
```

<!-- cargo-rdme end -->
