%%---10--|----20---|----30---|----40---|----50---|
-module(long_list).

-export([long_simple_list/0,
         long_list_with_comments/0,
         nested_list/0]).


long_simple_list() ->
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
     14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
     25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
     36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
     47, 48, 49, 50].


long_list_with_comments() ->
    [1, 2, 3, 4, 5,  % hi
     6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
     18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
     29, 30, 31, 32, 33, 34, 35, 36,  % hello
     37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
     48, 49, 50].


nested_list() ->
    [1,
     2,
     [3, 4, 5, 6, 7, 8, 9, 10],
     11,
     [12, 13, 14, 15, 16, 17, 18],
     [19,
      20,
      21,
      22,
      [23, 24, 25, 26, 27, 28],
      29,
      30,
      31,
      32],
     33,
     [34, 35, [36, 37], 38, 39],
     40,
     [41, 42, 43, 44],
     45,
     [46,
      47,
      48,
      49,
      [50, 51, 52, 53],
      54,
      55,
      [56, 57, 58, 59, 60, 61, 62, 63, 64],
      65,
      66]].
