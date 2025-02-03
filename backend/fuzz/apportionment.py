#!/usr/bin/env python3

from fractions import Fraction
from typing import Callable, List

# Seat apportionment algorithm based on https://github.com/squell/electionchecker

# Does not yet support the following features (which Abacus also does not yet support)
# - Drawing with lots: throws an exception if drawing with lots is needed to decide the final allocation
# - Absolute majority corrections 
# - National apportionment for the Dutch parliament and European Parliament (has different voting threshold)
# - Lijstuitputting

# Perform a seat apportionment, selecting D'Hondt or modified-Hamilton
# based on the number of seats, as Dutch law does for bodies.
def allocate(total_seats: int, votes: List[int]) -> List[int]:
    if total_seats >= 19:
        return allocate_per_average(total_seats, votes)
    else:
        return allocate_per_surplus(total_seats, votes)


# Perform a seat apportionment based on the given method.
# It is a **requirement** that the `criterion` algorithm will always rank a party that is
# eligible for at least one more seat above a party that doesn't.
def allocate_seats(total_seats: int, votes: List[int], criterion: Callable[[int, int], int], seats: List[int] | None = None) -> List[int]:
    total_votes = sum(votes)
    
    if seats is None:
        seats = [total_seats * v // total_votes for v in votes] # initialize with whole seats

    while sum(seats) < total_seats:
        # Compute how much each party deserves an extra seat
        qualities = [criterion(v,s) for v, s in zip(votes, seats)]
        max_quality = max(qualities)

        if max_quality < 0:
            # No parties deserve another seat according to the provided criterion
            return seats

        # Gather all parties that deserve an extra seat
        awarded = [i for i, q in enumerate(qualities) if q == max_quality]

        if sum(seats) + len(awarded) > total_seats:
            # Multiple parties are equally eligible, but we can not give them all a seat.
            raise Exception("Drawing with lots is needed!")

        # Award seats
        for i in awarded:
            seats[i] += 1

    return seats


# Perform a seat apportionment based on the D'Hondt method.
# This system is currently used in the Netherlands for regional councils least 19 seats or more.
def allocate_per_average(total_seats: int, votes: List[int]) -> List[int]:
    return allocate_seats(total_seats, votes, lambda v, s: Fraction(v, s + 1))


# Perform a seat apportionment based on the Hamilton method, with a
# voting threshold of 75% of a whole seat, and parties receiving a maximum of one extra seat.
# If seats remain after that, apportion the remainder of seats using D'Hondt, with
# parties again only receiving a maximum of one additional seat.
# This system is currently used in the Netherlands for bodies of less than 19 seats.
def allocate_per_surplus(total_seats: int, votes: List[int]) -> List[int]:
    total_votes = sum(votes)
    quota = Fraction(total_votes, total_seats) # the "kiesdeler"

    def has_surplus(v, s):
        return v >= s * quota

    def meets_threshold(v):
        return v >= Fraction(3, 4) * quota

    # Gives max. 1 extra seat per party.
    def surplus(v, s):
        return v - s * quota if has_surplus(v, s) and meets_threshold(v) else -1

    seats = allocate_seats(total_seats, votes, surplus)
    
    if sum(seats) < total_seats:
        # Gives max. 1 more extra seat per party.
        def quotient_with_threshold(v, s):
            if has_surplus(v, (s - 1) if meets_threshold(v) else s):
                return Fraction(v, s + 1)
            else:
                return -1

        seats = allocate_seats(total_seats, votes, quotient_with_threshold, seats=seats)
    
    return seats


# Test cases taken from https://github.com/kiesraad/abacus/blob/9648abc120c82322ea1a91d5ab4684ba09b91592/backend/src/apportionment/mod.rs#L531
if __name__ == "__main__":
    def expect_exception(total_seats, votes):
        try:
            allocate(total_seats, votes)
        except:
            return True
        else:
            return False

    # Some test cases for < 19 seats
    assert allocate(15, [480, 160, 160, 160, 80, 80, 80]) == [6, 2, 2, 2, 1, 1, 1]
    assert allocate(15, [540, 160, 160, 80, 80, 80, 60, 40]) == [7, 2, 2, 1, 1, 1, 1, 0]
    assert allocate(15, [808, 59, 58, 57, 56, 55, 54, 53]) == [12, 1, 1, 1, 0, 0, 0, 0]
    assert expect_exception(15, [540, 160, 160, 80, 80, 80, 55, 45]) # drawing of lots
    assert expect_exception(15, [500, 140, 140, 140, 140, 140]) # drawing of lots

    # # Some test cases for >= 19 seats
    assert allocate(25, [576, 288, 96, 96, 96, 48]) == [12, 6, 2, 2, 2, 1]
    assert allocate(23, [600, 302, 98, 99, 101]) == [12, 6, 1, 2, 2]
    assert expect_exception(23, [500, 140, 140, 140, 140, 140]) # drawing of lots
