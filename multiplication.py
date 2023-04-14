from concrete import fhe

@fhe.compiler({"s" : "encrypted", "b" : "encrypted"})
def mult(s, b):
    return s*b*s

def clean_part(s,b):
    print(s, b)
    CONFIGURATION = fhe.Configuration(
        enable_unsafe_features=True,
        use_insecure_key_cache=True,
        insecure_key_cache_location=".keys",
    )

    inputset = [ (1, 1) , (7,7) ]

    circuit = mult.compile(inputset, CONFIGURATION)

    res = circuit.encrypt_run_decrypt(s,b)

    print(circuit)
    print(res)
    

clean_part(7, 7)
