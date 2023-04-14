from concrete import fhe
import numpy as np
import time
import random

MAX_VALUE = 8
MIN_VALUE = 0
VALUE_BITWIDTH = 3

MAXLENGTH = 2**3

select_lut = fhe.LookupTable([0 for i in range(2**VALUE_BITWIDTH)] + [i for i in range(2**VALUE_BITWIDTH)])

def min(a, b):
    return np.minimum(a-b, 0) + b

def sum(list):
    r = 0
    for val in list:
        r += val
    return r

@fhe.compiler({"s" : "encrypted", "b" : "encrypted"})
def dark_market(s, b):
    sellVol = sum(s)
    buyVol = sum(b)
    transVol = min(sellVol, buyVol)

    leftVol = transVol

    for i in range(MAXLENGTH):
        #z_1 = (leftVol <= 0)
        #z_2 = (leftVol < s[i])
        #s[i] = ( ( leftVol - s[i] ) * z_2 + s[i] ) * (1 - z_1)
        s[i] = min(s[i], leftVol)
        leftVol -= s[i]

        #value_to_lookup = min(leftVol, s[i]) + 2**VALUE_BITWIDTH * (1 - (leftVol >= 0))
        #s[i] = select_lut[value_to_lookup]

    leftVol = transVol

    for i in range(MAXLENGTH):
        b[i] = min(b[i], leftVol)
        leftVol -= b[i]

    #Todo: return both lists at the same time
    return b

def clean_part(s,b):
    print(s, b)
    CONFIGURATION = fhe.Configuration(
        enable_unsafe_features=True,
        use_insecure_key_cache=True,
        insecure_key_cache_location=".keys",
    )

    inputset = [ ([1,2,3,0,0,0,0,0],[5,1,0,0,0,0,0,0]) , ([0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0]) ]

    circuit = dark_market.compile(inputset, CONFIGURATION)

    res = circuit.encrypt_run_decrypt(s,b)

    print(res)

s = [1,2,3] + [0] * (MAXLENGTH - 3)
b = [2,2] + [0] * (MAXLENGTH - 2)

clean_part(s, b)
res = dark_market(s, b)
print(res)