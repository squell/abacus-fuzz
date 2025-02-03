#![no_main]

use abacus::{
    apportionment::{seat_assignment as seat_allocation, ApportionmentError},
    data_entry::PoliticalGroupVotes,
};
use abacus_fuzz::FuzzedElectionSummary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (FuzzedElectionSummary, u16)| {
    let (data, added_votes) = data;
    let alloc = seat_allocation(data.seats, &data.election_summary);

    // Add some votes to a party
    let mut data = data;
    data.votes[0] += added_votes as u32;
    data.total_votes += added_votes as u32;

    let political_group_votes: Vec<PoliticalGroupVotes> = data
        .votes
        .iter()
        .enumerate()
        .map(|(index, votes)| {
            PoliticalGroupVotes::from_test_data_auto((index + 1) as u32, *votes, &[])
        })
        .collect();

    data.election_summary.voters_counts.poll_card_count = data.total_votes;
    data.election_summary
        .voters_counts
        .total_admitted_voters_count = data.total_votes;
    data.election_summary.votes_counts.votes_candidates_count = data.total_votes;
    data.election_summary.political_group_votes = political_group_votes;

    let new_alloc = seat_allocation(data.seats, &data.election_summary);

    match (alloc, new_alloc) {
        (Ok(alloc), Ok(new_alloc)) => {
            let seats_per_party = alloc.get_total_seats();
            let new_seats_per_party = new_alloc.get_total_seats();

            // The party with the extra votes should have as least as many seats as before
            let my_party_seats = seats_per_party[0];
            let new_my_party_seats = new_seats_per_party[0];
            assert!(
                new_my_party_seats >= my_party_seats,
                "{new_my_party_seats} is not greater or equal than {my_party_seats}\n{seats_per_party:?} -> {new_seats_per_party:?}",
            );
        }
        (Err(ApportionmentError::DrawingOfLotsNotImplemented), _) => {} // ignore DrawingOfLotsNotImplemented errors
        (_, Err(ApportionmentError::DrawingOfLotsNotImplemented)) => {} // ignore DrawingOfLotsNotImplemented errors
	_ => panic!()
    }
});
