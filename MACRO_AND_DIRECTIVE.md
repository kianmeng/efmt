How Macros and Directives are Handled
=====================================

Note that this document doesn't explain Erlang macros and directives themselves.
So if you want to know the detail of them, please refer to
[Erlang Reference Manual - 9. Preprocessor](https://www.erlang.org/doc/reference_manual/macros.html).


Macros
------

A unique point of `efmt` among various Erlang formatters is that it can handle macros correctly.

For example, `efmt` can format the following code containing unusual macros without errors.

### Original code

```erlang
-module(weird_macro).

-define(FOO, /).
-define(BAR, :format().
-define(baz(A), A).
-define(qux, -> [1, 2, 3], [).
-define(quux, )], [2,).
-define(a(A, B), A).

-export([?baz(?baz(main))?FOO 0]).

hello(A)->io?BAR "hello ~p\n",[A]).
main()?a(?qux a, b), c],[1, hello(world?quux 3].
```

### Formatted code

`$ efmt weird_macro.erl`

```erlang
-module(weird_macro).

-define(FOO, /).
-define(BAR, :format().
-define(baz(A), A).
-define(qux, -> [1, 2, 3], [).
-define(quux, )], [2,).
-define(a(A, B), A).

-export([?baz(?baz(main))?FOO 0]).

hello(A) ->
    io?BAR "hello ~p\n", [A]).
main() ?a(?qux a, b)
    , c],
    [1, hello(world?quux
    3].
```

To make it possible, during the parse phase, `efmt` collects macro definitions (i.e., `-define` directives) and expands macro calls (i.e., `?MACRO_NAME`) to build an abstract syntax tree-like structure from the input text.
Then, during the format phase, `efmt` traverses the tree and emits the formatted text representing each tree node.
When it visits tree nodes expanded from a macro, the formatted text of the original macro call is emitted instead.


`-include` and `-include_lib` Directives
----------------------------------------

To collect macro definitions needed to parse an input file, `efmt` try to process `-include` and `-include_lib` directives in the file as much as possible as the Erlang preprocessor does.
If `efmt` fails to find the include target file, it just ignores the directive.
Note that when `efmt` encounters an unknown macro, that might be defined in the ignored file, during the parse phase, the macro is replaced with the `'EFMT_DUMMY'` atom token.

Resolving the file path specified by `-include_lib` is more complicated than `-include` as the first path component can be the name of an Erlang application rather than a plain directory name.
So, `efmt` invokes `erl -noshell -eval 'io:format("~s", [code:lib_dir($APP_NAME)]), halt().'` command to try to resolve the application directory path.

### Include cache

The macro definitions collected during processing a `-include` or `-include_lib` directive is cached as a JSON file under `$PWD/.efmt/cache/v0/` directory (`v0` is the current cache format version).
The cache file is used when processing the same include target file next time to reduce the overhead of parsing the whole file from scratch.

### `efmt` options

Note that `efmt` provides some options to control how to handle those directives as follows:
- `--include-search-dir` (`-I` in short)
- `--include-cache-dir`
- `--disable-include`
- `--disable-include-cache`

Please run `$ efmt --help` to see the descriptions of those options.


Flow Control Directives
-----------------------

`efmt` doesn't recognize the following directives relating to flow control:
- `-undef(Macro)`
- `-ifdef(Macro)`
- `-ifndef(Macro)`
- `-else`
- `-endif`
- `-if(Condition)`
- `-elif(Condigion)`

Those are treated as ordinal attributes.
That is, both the "then" and "else" branch forms are always processed.
In many cases, this behavior doesn't cause a problem.
However, if either of the branches contains corrupted code, `efmt` would fail to format the file (see the example code below).

```erlang
-define(FOO, foo).

-ifdef(FOO).

%% As the `FOO` macro is defined in the same file, the Erlang compiler always evaluates this branch.
foo() -> ok.

-else.

%% On the other hand, this branch is always removed by the preprocessor.
%% But, `efmt` tries to parse this branch. Then it fails as the following function declaration is invalid.
foo()

-endif.
```
