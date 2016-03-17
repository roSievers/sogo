# This script should calculate all possible winning states.
# In particluar, it should be able to verify if a set of rules generates the correct set of states.

# A state only contains binary information, so we could represent them using a
# binary number. (A python long with 64 bits.) This allows a compact storage and
# easy hashing.

# But to work with them it needs a structure with nice addition.
# (Int, Int, Int) should be a nice match for that.

def vector_addition(v1, v2):
    return (v1[0]+v2[0], v1[1]+v2[1], v1[2]+v2[2])

# Generally I want to keep this script using a generator style.
# There generators are passed around as a function returning a generator.
# This way I can create a new version of the generator whenever I want.

def gen_vector_addition(g1, g2):
    def closure():
        for v1 in g1():
            for v2 in g2():
                yield vector_addition(v1, v2)
    return closure

def any_point():
    for x in xrange(4):
        for y in xrange(4):
            for z in xrange(4):
                yield (x, y, z)

def any_direction():
    for x in xrange(-1, 2):
        for y in xrange(-1, 2):
            for z in xrange(-1, 2):
                if x == y == z == 0:
                    # The zero vector is not a direction.
                    continue
                yield (x, y, z)

# A victory state is a particular unordered combination of several positions.
# This is represented in python using a dictionary without values. {Int : (Int, Int, Int)}

# This function takes a generator and returns state object for further inspection.
def gather_points(g):
    result = {}
    for e in g():
        result[flat_coordinate(*e)] = e
    return result

def flat_coordinate(x, y, z):
    return x + 4*y + 16*z

# We can now perform some sanity checks on the victory states.
# These are meant to be applied via a filter.

def check_bounding_box(v):
    for coordinate in v.values():
        for i in xrange(3):
            if not (0 <= coordinate[i] <= 3):
                return False
    return True

def check_size(size):
    def closure(v):
        return len(v) == size
    return closure

# After the victory state is judged as valid, it is flattened to a 64 bit integer
# and stored in a hash map to avoid double counting.

def gather_states(g):
    result = {}
    for e in g():
        result[flat_state(e)] = None
    return result

def flat_state(v):
    result = 0
    for key in v:
        result += 2**key
    return result


# Now, let's apply all those functions we build!
# Case one, lines!

def point(p):
    def closure():
        yield p
    return closure

def line(v, length):
    def closure():
        p = (0, 0, 0)
        for i in xrange(length):
            yield p
            p = vector_addition(p, v)
    return closure

def all_lines():
    for p in any_point():
        for v in any_direction():
            yield gen_vector_addition(point(p), line(v, 4))

def legal_lines():
    for sub_g in all_lines():
        victory_state = gather_points(sub_g)
        #print victory_state
        if not check_bounding_box(victory_state):
            continue
        if not check_size(4)(victory_state):
            continue
        yield victory_state

def exhaust_state_generator(g):
    all_states = gather_states(g);
    return all_states

# Turns out this actually works really well.
# exhaust_state_generator(all_lines) does return an object with 76 entries.

# Next up, we can analyse the possible parallelograms.
def all_parallelograms():
    for p in any_point():
        for v in any_direction():
            axis1 = gen_vector_addition(point(p), line(v, 3))
            for w in any_direction():
                yield gen_vector_addition(axis1, line(w, 2))

def legal_parallelograms():
    for sub_g in all_parallelograms():
        victory_state = gather_points(sub_g)
        #print victory_state
        if not check_bounding_box(victory_state):
            continue
        if not check_size(6)(victory_state):
            continue
        yield victory_state

print """Use
> a = exhaust_state_generator(legal_parallelograms)
> len(a)
to find the number of legal parallelograms."""

# Now we can use this list of all parallelograms to check an algorithm with
# generates all the parallelograms in a sensible fashion.
# Or we just place them in a long list of 1020 entries :P

a = exhaust_state_generator(legal_lines)
print a.keys()
