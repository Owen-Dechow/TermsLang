class Loopy:
    times_run: int

    def __init__(self):
        self.times_run = 0

    def loop(self, n):
        for i in range(n):
            i + i

        self.times_run += 1
        print("Times Run:", self.times_run)


if __name__ == "__main__":
    loopy = Loopy()
    for i in range(10):
        loopy.loop(10000000)
