-module(long_string).

-export([hello/1]).

hello(A) ->
    io:format("> hello hello hello hello hello: ~p", [A]).
