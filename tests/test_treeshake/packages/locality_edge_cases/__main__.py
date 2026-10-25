from locality_edge_cases import shadow_except, shadow_for, shadow_match, shadow_with, use_choice

if __name__ == "__main__":
    print(shadow_for())
    print(shadow_with())
    print(shadow_except())
    print(shadow_match({"key": "v"}))
    print(use_choice())
