from concrete import fhe
import numpy as np
import time
import random

VALUE_BITWIDTH = 3
MAX_VALUE = 2**VALUE_BITWIDTH - 1
MIN_VALUE = 0

MAXLENGTH = 2**3

select_lut = fhe.LookupTable([0 for i in range(2**VALUE_BITWIDTH)] + [i for i in range(2**VALUE_BITWIDTH)])

def min(a, b):
    return np.minimum(a-b, 0) + b

@fhe.compiler({"a" : "encrypted", "b" : "encrypted"})
def add(a ,b):
    for i in range(15):
        a = (a+b) % 64
    return a

def sum(list):
    r = 0
    for val in list:
        r += val
    return r

@fhe.compiler({"s" : "encrypted", "b" : "encrypted"})
def dark_market(s, b):
    sellVol = np.sum(s)
    buyVol = np.sum(b)
    transVol = min(sellVol, buyVol)

    leftVol = transVol

    for i in range(MAXLENGTH):
        #z_1 = (leftVol <= 0)
        #z_2 = (leftVol < s[i])
        #s[i] = ( ( leftVol - s[i] ) * z_2 + s[i] )
        s[i] = min(s[i], leftVol)
        leftVol -= s[i]
        #value_to_lookup = min(leftVol, s[i]) + 2**VALUE_BITWIDTH * (1 - (leftVol >= 0))
        #s[i] = select_lut[value_to_lookup]

    leftVol = transVol

    for i in range(MAXLENGTH):
        #z_2 = (leftVol < s[i])
        #b[i] = ( ( leftVol - b[i] ) * z_2 + b[i] )
        b[i] = min(b[i], leftVol)
        leftVol -= b[i]
    return np.concatenate((s, b))

def clean_part(s,b):
    print(s, b)
    print("")
    CONFIGURATION = fhe.Configuration(
        dataflow_parallelize = True,
        auto_parallelize = True,
        enable_unsafe_features=True,
        use_insecure_key_cache=True,
        insecure_key_cache_location=".keys",
    )

    inputset = [([random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)], [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)]) ,
                ([MAX_VALUE for i in range(MAXLENGTH)],[MAX_VALUE for i in range(MAXLENGTH)]),
                ([MAX_VALUE for i in range(MAXLENGTH)],[MAX_VALUE for i in range(MAXLENGTH)])]

    add_input = [(random.randint(0,63), random.randint(0, 63)) for i in range(100)]
    add_circuit = add.compile(add_input, CONFIGURATION)

    circuit = dark_market.compile(inputset, CONFIGURATION)
    #a = circuit.encrypt(s, b)
    c = add_circuit.encrypt(60, 60)
    start = time.time()
    #b = circuit.run(a)
    d = add_circuit.run(c)
    end = time.time()
    #res = circuit.decrypt(b)
    res = add_circuit.decrypt(d)
    #res = list(circuit.encrypt_run_decrypt(s,b))
    print("The encrypted result:")
    print(res)
    print("The time it took:")
    print(end - start)
    #print(circuit)

#s = [1,2,3] + [0] * (MAXLENGTH - 3)
#b = [2,2] + [0] * (MAXLENGTH - 2)
s = [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)]
b = [random.randint(0, MAX_VALUE) for i in range(MAXLENGTH)]
print("Running algorithm on:")
clean_part(s, b)
print("")
res = list(dark_market(s, b))
print(res)