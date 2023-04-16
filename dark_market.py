from concrete import fhe
import numpy as np
import time
import random

VALUE_BITWIDTH = 4
MAX_VALUE = 2**VALUE_BITWIDTH - 1
MIN_VALUE = 0

MAXLENGTH = 2**3

select_lut = fhe.LookupTable([0 for i in range(2**VALUE_BITWIDTH)] + [i for i in range(2**VALUE_BITWIDTH)])

def min(a, b):
    return np.minimum(a-b, 0) + b

def sum(list):
    r = 0
    for val in list:
        r += val
    return r

@fhe.compiler({"s" : "encrypted", "b" : "encrypted", "r" : "encrypted"})
def dark_market(s, b, r):
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
        r[i] = s[i]

        #value_to_lookup = min(leftVol, s[i]) + 2**VALUE_BITWIDTH * (1 - (leftVol >= 0))
        #s[i] = select_lut[value_to_lookup]

    leftVol = transVol

    for i in range(MAXLENGTH):
        b[i] = min(b[i], leftVol)
        leftVol -= b[i]
        r[i+MAXLENGTH] = b[i]

    #Todo: return both lists at the same time
    return r

def clean_part(s,b, r):
    print(s, b)
    CONFIGURATION = fhe.Configuration(
        enable_unsafe_features=True,
        use_insecure_key_cache=True,
        insecure_key_cache_location=".keys",
    )

    inputset = [([random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)], [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)], [0 for i in range(2*MAXLENGTH)]) , 
                ([MAX_VALUE for i in range(MAXLENGTH)],[MAX_VALUE for i in range(MAXLENGTH)], [0 for i in range(2*MAXLENGTH)]),
                ([MAX_VALUE for i in range(MAXLENGTH)],[MAX_VALUE for i in range(MAXLENGTH)], [0 for i in range(2*MAXLENGTH)])]

    circuit = dark_market.compile(inputset, CONFIGURATION)

    res = list(circuit.encrypt_run_decrypt(s,b,r))

    print(res)
    print(circuit)

#s = [1,2,3] + [0] * (MAXLENGTH - 3)
#b = [2,2] + [0] * (MAXLENGTH - 2)
s = [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)]
b = [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)]
r = [0 for i in range(2*MAXLENGTH)]
clean_part(s, b, r)
res = dark_market(s, b, r)
print(res)