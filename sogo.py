from random import choice

# A gamestate of Sogo is an array of length 16 containing arrays of variable
# length (from 0 to 4)

# What is a line?
# How many lines are there?
# There are strait lines, 16*3 of them.
# There are diagonal lines, 8*3 of them.
# There are spacial diagonals, 4 of them.
# This makes a total of 76
# I guess we would want a list of points,
# And a reference from the points to the avaliable lines
# But all of this is immutable data that only needs to be computed once and
# may be shared between multiple models of the game.

class Point(object):
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z
        self.lines = []
        self.flat = flatten(x, y, z)
    def __repr__(self):
        return "Point({0}, {1}, {2})".format(self.x, self.y, self.z)

def flatten(x, y, z):
    return x + 4*y + 16*z

# generate a list of all points
_pointBox = []
for z in xrange(4):
    for y in xrange(4):
        for x in xrange(4):
            _pointBox.append(Point(x, y, z))

def points(x, y, z):
    return _pointBox[flatten(x, y, z)]

_idCounter = 0
class Line(object):
    def __init__(self, x, y, z, dx, dy, dz):
        self.points = [points(x+i*dx, y+i*dy, z+i*dz) for i in xrange(4)]
        for point in self.points:
            point.lines.append(self)
        global _idCounter
        self.id = _idCounter
        _idCounter += 1
    def __repr__(self):
        return "Line({0}, {1}, {2}, {3})".format(*self.points)

# generate a list of all lines
_lineBox = []
# lines in only one dimension
for a in xrange(4):
    for b in xrange(4):
        _lineBox.append(Line(a, b, 0, 0, 0, 1))
        _lineBox.append(Line(0, a, b, 1, 0, 0))
        _lineBox.append(Line(b, 0, a, 0, 1, 0))
# diagonals in two dimensions
for a in xrange(4):
    _lineBox.append(Line(0, 0, a, 1, 1, 0))
    _lineBox.append(Line(0, 3, a, 1,-1, 0))
    _lineBox.append(Line(a, 0, 0, 0, 1, 1))
    _lineBox.append(Line(a, 0, 3, 0, 1,-1))
    _lineBox.append(Line(0, a, 0, 1, 0, 1))
    _lineBox.append(Line(3, a, 0,-1, 0, 1))
# space diagonals in all three dimensions
_lineBox.append(Line(0, 0, 0, 1, 1, 1))
_lineBox.append(Line(0, 3, 0, 1,-1, 1))
_lineBox.append(Line(3, 3, 0,-1,-1, 1))
_lineBox.append(Line(3, 0, 0,-1, 1, 1))

# Now that we have some abstract definitions, how about an actual gamestate?

def resetGamestate():
    global gamestate
    gamestate = ([None] * len(_pointBox), [0] * len(_lineBox))
resetGamestate()

# Ideally the gamestate would be a struct
# struct GameState {
#     points : [Point, 64],
#     lines  : [Line, 75]  // something something mutable?
# }

def zValue(x, y):
    for z in xrange(4):
        if gamestate[0][flatten(x, y, z)] is None:
            return z
    return False

def playAt(x, y, token):
    result = None
    z = zValue(x, y)
    if z is False:
        raise ValueError("Column ({0},{1}) has no space left.".format(x, y))
    flatCoordinate = flatten(x, y, z)
    gamestate[0][flatCoordinate] = token
    for line in _pointBox[flatCoordinate].lines:
        gamestate[1][line.id] += 1
        if gamestate[1][line.id] == 4:
            if checkVictory(line):
                print "someone has won the game"
                result = "Victory"
    return result

def checkVictory(line):
    return checkEqual(map(lambda p: gamestate[0][p.flat], line.points))

# I feel like the lines need a kind of state which should be
# enum LineState {  // assuming I implement this in Rust at some point.
#     Empty,
#     White(i8),
#     Black(i8), // or maybe even better:
#     Pure { color: PlayerColor, count: i8 },
#     Mixed
# }

def checkEqual(iterator):
    # https://stackoverflow.com/questions/3844801/check-if-all-elements-in-a-list-are-identical
    # By kennytm.
    try:
        iterator = iter(iterator)
        first = next(iterator)
        return all(first == rest for rest in iterator)
    except StopIteration:
        return True

def charStr(c):
    if c is None:
        return ""
    elif c is 0:
        return "O"
    elif c is 1:
        return "X"
    else:
        return "?"

def displayBoard():
    for z in xrange(3, -1, -1):
        print "Layer {0}:".format(z)
        for y in xrange(4):
            print "".join([charStr(gamestate[0][flatten(x, y, z)]) for x in xrange(4)])

def getLegalMoves():
    z = 4
    legal = []
    for x in xrange(4):
        for y in xrange(4):
            if zValue(x, y) is not False:
                legal.append((x, y))
    return legal

def randomPlayout():
    currentPlayer = 0
    running = True
    count = 0
    while running:
        x, y = choice(getLegalMoves())
        r = playAt(x, y, currentPlayer)
        currentPlayer = 1 - currentPlayer
        count += 1
        if r is not None:
            running = False
    return count

def getSample(size):
    sample = []
    for i in xrange(size):
        print "Running game {0}".format(i)
        resetGamestate()
        n = randomPlayout()
        sample.append(n)
    return sample

def printToCopy(array):
    for i in array:
        print "{0},".format(i),


def randomPlayer(color):
    def play():
        return choice(getLegalMoves())
    play.color = color
    return play

def zeroStackPlayer(color):
    def play():
        if zValue(0, 0) is not False:
            return (0, 0)
        else:
            return choice(getLegalMoves())
    play.color = color
    return play

def pitchPlayers(player1, player2):
    resetGamestate()
    players = [player1, player2]
    running = True
    count = 0
    currentPlayer = 0
    while running:
        x, y = players[currentPlayer]()
        r = playAt(x, y, players[currentPlayer].color)
        currentPlayer = 1 - currentPlayer
        count += 1
        if r is not None:
            running = False
    return count
