%%---10--|----20---|----30---|----40---|----50---|
-module(record).

-record(foo, {
          a,
          b,
          c
         }).

-type foo() :: #foo{
                 a :: integer(),
                 b :: term(),
                 c :: any()
                }.


main() ->
    X0 = #foo{},
    X1 = #foo{a = 1, b = 2},
    X2 = #foo{
           a = 1,
           b = 2,
           c = 3
          },
    X3 = #foo{
           a = 1000000000000000,
           b = 2000000000000000
          },
    X4 = X3#foo{
           a = 1,
           b = 2,
           c = 3
          }.
