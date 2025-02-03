#![no_main]

use abacus::apportionment::{seat_assignment as seat_allocation, ApportionmentError};
use abacus_fuzz::FuzzedElectionSummary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (FuzzedElectionSummary, u16)| {
    let (data, added_seats) = data;
    let seats = data.seats + 18; // apportionment for < 19 seats is not monotonic
    let new_seats = seats + u32::from(added_seats);

    match (
        seat_allocation(seats, &data.election_summary),
        seat_allocation(new_seats, &data.election_summary),
    ) {
        (Ok(alloc), Ok(new_alloc)) => {
            let seats_per_party = alloc.get_total_seats();
            let new_seats_per_party = new_alloc.get_total_seats();

            // House monotonicity:
            // The number of seats given to any party will not decrease if the number of seats increases (when votes stay the same)
            assert!(
                new_seats_per_party.iter().ge(seats_per_party.iter()),
                "{new_seats_per_party:?} ({new_seats} seats) is not greater or equal than {seats_per_party:?} ({seats} seats)",
            );
        }
        (Err(ApportionmentError::DrawingOfLotsNotImplemented), _) => {} // ignore DrawingOfLotsNotImplemented errors
        (_, Err(ApportionmentError::DrawingOfLotsNotImplemented)) => {} // ignore DrawingOfLotsNotImplemented errors
	_ => panic!()
    }
});
