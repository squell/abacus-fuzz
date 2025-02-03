#![no_main]

use abacus::apportionment::{seat_assignment as seat_allocation, ApportionmentError};
use abacus_fuzz::FuzzedElectionSummary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: FuzzedElectionSummary| {
    match seat_allocation(data.seats, &data.election_summary) {
        Ok(alloc) => {
            let total_votes = data.total_votes;
            let votes_per_party = data.votes;
            assert_eq!(total_votes, votes_per_party.iter().sum::<u32>());

            let total_seats = data.seats;
            let seats_per_party = alloc.get_total_seats();
            assert_eq!(
                total_seats,
                seats_per_party.iter().sum::<u32>(),
                "{seats_per_party:?}"
            );

            // Lower bound:
            // every party deserves at least the percentage of seats relative to the percentage of votes (rounded down)
            let seats_lower_bound = votes_per_party
                .iter()
                .map(|v| u64::from(total_seats) * u64::from(*v) / u64::from(total_votes))
                .map(|x| x.try_into().unwrap())
                .collect::<Vec<u32>>();
            assert!(
                seats_per_party.iter().ge(seats_lower_bound.iter()),
                "{:?} is not greater or equal than {:?}",
                seats_per_party,
                seats_lower_bound,
            );

            // Upper bound:
            // let extra = std::cmp::max(total_seats / 5, 2); // max 20% extra (incorrect)
            let extra = seats_per_party.len() as u64 - 1; // every other party has the ability to add 1 "restzetel"
            let seats_upper_bound = votes_per_party
                .iter()
                .map(|v| {
                    ((u64::from(total_seats) * u64::from(*v)) as f64 / (total_votes as f64)).ceil() as u64
                        + extra
                })
                .map(|x| x.try_into().unwrap())
                .collect::<Vec<u32>>();
            assert!(
                seats_per_party.iter().le(seats_upper_bound.iter()),
                "{:?} is not less or equal than {:?}\nextra: {}\nrest seats: {:?}",
                seats_per_party,
                seats_upper_bound,
                extra,
                alloc.get_rest_seats()
            );

            for (votes, seats) in votes_per_party.iter().zip(seats_per_party.iter()) {
                for (other_votes, other_seats) in votes_per_party.iter().zip(seats_per_party.iter())
                {
                    if votes == other_votes {
                        // Balancedness: for parties with an equal number of votes, the allocation should differ with at most 1 seat
                        // Since drawing of lots is not (yet) implemented in Abacus, the difference should currently always be 0
                        let difference = seats.abs_diff(*other_seats);
                        assert!(difference <= 1,
                            "Difference of parties with an equal number of votes should be at most 1 (was {difference})\n\
                            {votes_per_party:?}\n{seats_per_party:?}");
                    }

                    if votes > other_votes {
                        // Concordance: parties with more votes should have at least as many seats
                        assert!(seats >= other_seats,
                            "Party with more votes should have at least as many seats ({votes} vs {other_votes} votes, {seats} vs {other_seats} seats),\n\
                            {votes_per_party:?}\n{seats_per_party:?}");
                    }
                }
            }
        }
        Err(ApportionmentError::DrawingOfLotsNotImplemented) => {} // ignore
	_ => panic!()
    }
});
