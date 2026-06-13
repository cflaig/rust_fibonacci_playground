def print_fibonacci_sequence(limit):
    a, b = 0, 1
    for i in range(1, limit + 1):
        print(f"F_{i:<2} = {b}")
        print(f"Number bits: {b.bit_length()}")
        # Aktualisiere die Werte für den nächsten Schritt
        a, b = b, a + b

# Ausführen des Skripts für die Zahlen 1 bis 99
print_fibonacci_sequence(9210)
