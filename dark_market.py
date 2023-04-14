import concrete.numpy as cnp
import numpy as np
import time
import random

MAX_VALUE = 8
MIN_VALUE = 0
VALUE_BITWIDTH = 3

MAXN = 8
MAXM = 8

select_lut = cnp.LookupTable([0 for i in range(2**VALUE_BITWIDTH)] + [i for i in range(2**VALUE_BITWIDTH)])

def min(a, b):
    return np.minimum(a-b, 0) + b

@cnp.compiler({"s" : "encrypted", "b" : "encrypted", "n" : "clear", "m" : "clear"})
def dark_market(s, b, n, m):
    sellVol = 0
    for i in range(3):
        sellVol = sellVol + s[i]

    buyVol = 0
    for i in range(2):
        buyVol = buyVol + b[i]

    transVol = min(sellVol, buyVol)

    leftVol = transVol

    for i in range(3):
        value_to_lookup = min(leftVol, s[i]) + 2**VALUE_BITWIDTH * (1 - (leftVol >= 0))
        s[i] = select_lut[value_to_lookup]

    return s

CONFIGURATION = cnp.Configuration(
    enable_unsafe_features=True,
    use_insecure_key_cache=True,
    insecure_key_cache_location=".keys",
)

inputset = [([1,2,3],[3,2],3,2),([0,0,0],[0,0],3,2)]

circuit = dark_market.compile(inputset, CONFIGURATION)

res = circuit.encrypt_run_decrypt([1,2,3],[3,2],3,2)
print(res)