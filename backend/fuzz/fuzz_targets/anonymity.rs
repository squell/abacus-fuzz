#![no_main]

use abacus::{
    apportionment::{seat_assignment as seat_allocation, ApportionmentError},
    data_entry::PoliticalGroupVotes,
};
use abacus_fuzz::FuzzedElectionSummary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (FuzzedElectionSummary, Vec::<usize>)| {
    let (mut data, mut random_order) = data;
    let alloc = seat_allocation(data.seats, &data.election_summary);

    if random_order.len() == 0 {
        random_order.push(42);
    }

    // Randomly shuffle the party votes based on random_order
    let mut reorder = data.votes.iter().copied().enumerate().collect::<Vec<_>>();
    reorder.sort_by_key(|(i, _)| random_order[*i % random_order.len()]);

    let new_votes = reorder.iter().map(|(_, v)| *v).collect::<Vec<_>>();

    data.election_summary.political_group_votes = new_votes
        .iter()
        .enumerate()
        .map(|(index, votes)| {
            PoliticalGroupVotes::from_test_data_auto((index + 1) as u32, *votes, &[])
        })
        .collect();

    let new_alloc = seat_allocation(data.seats, &data.election_summary);

    match (alloc, new_alloc) {
        (Ok(alloc), Ok(new_alloc)) => {
            let seats_per_party = alloc.get_total_seats();

            // Reorder seat allocation to match the reordered parties
            let reordered = reorder
                .iter()
                .map(|(i, _)| seats_per_party[*i])
                .collect::<Vec<_>>();

            let new_seats_per_party = new_alloc.get_total_seats();

            // The allocation should be the same, but in the new order
            assert!(
                reordered.iter().eq(new_seats_per_party.iter()),
                "{reordered:?} (was {seats_per_party:?}) is not equal to {new_seats_per_party:?}\n{reorder:?}",
            );
        }
        (Err(ApportionmentError::DrawingOfLotsNotImplemented), _) => {} // ignore
        (_, Err(ApportionmentError::DrawingOfLotsNotImplemented)) => {} // ignore
	_ => panic!("error in fuzz test?")
    }
});
