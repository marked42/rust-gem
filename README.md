# rust-gem

## practices

1. custom error
1. custom Result type
1. readable type alias
1. builder pattern from complicated construction logic
1. performance recording and benchmark
1. organize into lib and mods
1. unit test (assert_cmd)
1. logging for easy debug

## TODO:

1. grep
   1. remove clone, use &str instead of String
   1. `cargo run -- line -lc` read std in, 'a line test b\n', std in blocked instead of output match
