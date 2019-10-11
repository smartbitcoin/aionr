use super::Header;
use super::U256;
use super::RewardsCalculator;
use super::UnityEngineParams;
use super::DifficultyCalc;
use super::SealType;

#[test]
fn test_calculate_rewards_number1() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::from(259200),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::from(748994641621655092u64),
        rampup_end_value: U256::from(1497989283243310185u64),
        lower_block_reward: U256::from(748994641621655092u64),
        upper_block_reward: U256::from(1497989283243310185u64),
        difficulty_bound_divisor: U256::from(1u64),
        difficulty_bound_divisor_unity: 1u64,
        minimum_difficulty: U256::zero(),
        block_time_lower_bound: 0u64,
        block_time_upper_bound: 0u64,
        block_time_unity: 0u64,
    };
    let calculator = RewardsCalculator::new(&params, None, U256::from(0));
    let mut header = Header::default();
    header.set_number(1);
    assert_eq!(
        calculator.calculate_reward(&header),
        U256::from(748997531261476163u64)
    );
}

#[test]
fn test_calculate_rewards_number10000() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::from(259200),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::from(748994641621655092u64),
        rampup_end_value: U256::from(1497989283243310185u64),
        lower_block_reward: U256::from(748994641621655092u64),
        upper_block_reward: U256::from(1497989283243310185u64),
        difficulty_bound_divisor: U256::from(1u64),
        difficulty_bound_divisor_unity: 1u64,
        minimum_difficulty: U256::zero(),
        block_time_lower_bound: 0u64,
        block_time_upper_bound: 0u64,
        block_time_unity: 0u64,
    };
    let calculator = RewardsCalculator::new(&params, None, U256::from(0));
    let mut header = Header::default();
    header.set_number(10000);
    assert_eq!(
        calculator.calculate_reward(&header),
        U256::from(777891039832365092u64)
    );
}

#[test]
fn test_calculate_rewards_number259200() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::from(259200),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::from(748994641621655092u64),
        rampup_end_value: U256::from(1497989283243310185u64),
        lower_block_reward: U256::from(748994641621655092u64),
        upper_block_reward: U256::from(1497989283243310185u64),
        difficulty_bound_divisor: U256::from(1u64),
        difficulty_bound_divisor_unity: 1u64,
        minimum_difficulty: U256::zero(),
        block_time_lower_bound: 0u64,
        block_time_upper_bound: 0u64,
        block_time_unity: 0u64,
    };
    let calculator = RewardsCalculator::new(&params, None, U256::from(0));
    let mut header = Header::default();
    header.set_number(259200);
    assert_eq!(
        calculator.calculate_reward(&header),
        U256::from(1497989283243258292u64)
    );
}

#[test]
fn test_calculate_rewards_number300000() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::from(259200),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::from(748994641621655092u64),
        rampup_end_value: U256::from(1497989283243310185u64),
        lower_block_reward: U256::from(748994641621655092u64),
        upper_block_reward: U256::from(1497989283243310185u64),
        difficulty_bound_divisor: U256::from(1u64),
        difficulty_bound_divisor_unity: 1u64,
        minimum_difficulty: U256::zero(),
        block_time_lower_bound: 0u64,
        block_time_upper_bound: 0u64,
        block_time_unity: 0u64,
    };
    let calculator = RewardsCalculator::new(&params, None, U256::from(0));
    let mut header = Header::default();
    header.set_number(300000);
    assert_eq!(
        calculator.calculate_reward(&header),
        U256::from(1497989283243310185u64)
    );
}

#[test]
fn test_calculate_difficulty_first_pos() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::zero(),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::zero(),
        rampup_end_value: U256::zero(),
        lower_block_reward: U256::zero(),
        upper_block_reward: U256::zero(),
        difficulty_bound_divisor: U256::from(2048u64),
        difficulty_bound_divisor_unity: 20u64,
        minimum_difficulty: U256::from(16),
        block_time_lower_bound: 5u64,
        block_time_upper_bound: 15u64,
        block_time_unity: 10u64,
    };
    let calculator = DifficultyCalc::new(&params, Some(0u64), Some(U256::from(2000000000u64)));
    let mut parent_header = Header::default();
    parent_header.set_timestamp(1524538000u64);
    parent_header.set_difficulty(U256::from(1));
    parent_header.set_number(3);
    let mut grand_parent_header = Header::default();
    grand_parent_header.set_timestamp(1524528000u64);
    grand_parent_header.set_number(2);
    let mut great_grand_parent_header = Header::default();
    great_grand_parent_header.set_timestamp(1524518000u64);
    great_grand_parent_header.set_number(1);
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(2000000000));
}

#[test]
fn test_calculate_difficulty() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::zero(),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::zero(),
        rampup_end_value: U256::zero(),
        lower_block_reward: U256::zero(),
        upper_block_reward: U256::zero(),
        difficulty_bound_divisor: U256::from(2048u64),
        difficulty_bound_divisor_unity: 20u64,
        minimum_difficulty: U256::from(16),
        block_time_lower_bound: 5u64,
        block_time_upper_bound: 15u64,
        block_time_unity: 10u64,
    };
    let calculator = DifficultyCalc::new(&params, Some(0u64), Some(U256::from(2000000000u64)));
    let mut parent_header = Header::default();
    parent_header.set_timestamp(1524538000u64);
    parent_header.set_difficulty(U256::from(1));
    parent_header.set_number(3);
    let mut grand_parent_header = Header::default();
    grand_parent_header.set_timestamp(1524528000u64);
    grand_parent_header.set_number(2);
    grand_parent_header.set_seal_type(SealType::PoS);
    let mut great_grand_parent_header = Header::default();
    great_grand_parent_header.set_timestamp(1524518000u64);
    great_grand_parent_header.set_number(1);
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(16));
}

#[test]
fn test_calculate_difficulty2() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::zero(),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::zero(),
        rampup_end_value: U256::zero(),
        lower_block_reward: U256::zero(),
        upper_block_reward: U256::zero(),
        difficulty_bound_divisor: U256::from(2048u64),
        difficulty_bound_divisor_unity: 20u64,
        minimum_difficulty: U256::from(16),
        block_time_lower_bound: 5u64,
        block_time_upper_bound: 15u64,
        block_time_unity: 10u64,
    };
    let calculator = DifficultyCalc::new(&params, Some(10u64), Some(U256::from(2000000000u64)));
    let mut parent_header = Header::default();
    parent_header.set_timestamp(1524528030u64);
    parent_header.set_difficulty(U256::from(2000));
    parent_header.set_number(3);
    let mut grand_parent_header = Header::default();
    grand_parent_header.set_timestamp(1524528010u64);
    grand_parent_header.set_number(2);
    grand_parent_header.set_difficulty(U256::from(2000));
    grand_parent_header.set_seal_type(SealType::PoS);
    let mut great_grand_parent_header = Header::default();
    great_grand_parent_header.set_timestamp(1524528000u64);
    great_grand_parent_header.set_number(1);
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(1999));

    // Unity difficulty rule
    let calculator = DifficultyCalc::new(&params, Some(0u64), Some(U256::from(2000000000u64)));
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(1904));
}

#[test]
fn test_calculate_difficulty3() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::zero(),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::zero(),
        rampup_end_value: U256::zero(),
        lower_block_reward: U256::zero(),
        upper_block_reward: U256::zero(),
        difficulty_bound_divisor: U256::from(2048u64),
        difficulty_bound_divisor_unity: 20u64,
        minimum_difficulty: U256::from(16),
        block_time_lower_bound: 5u64,
        block_time_upper_bound: 15u64,
        block_time_unity: 10u64,
    };
    let calculator = DifficultyCalc::new(&params, Some(10u64), Some(U256::from(2000000000u64)));
    let mut parent_header = Header::default();
    parent_header.set_timestamp(1524528020u64);
    parent_header.set_difficulty(U256::from(3000));
    parent_header.set_number(3);
    let mut grand_parent_header = Header::default();
    grand_parent_header.set_timestamp(1524528010u64);
    grand_parent_header.set_number(2);
    grand_parent_header.set_difficulty(U256::from(3000));
    grand_parent_header.set_seal_type(SealType::PoS);
    let mut great_grand_parent_header = Header::default();
    great_grand_parent_header.set_timestamp(1524528005u64);
    great_grand_parent_header.set_number(1);
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(3000));

    // Unity difficulty rule
    let calculator = DifficultyCalc::new(&params, Some(0u64), Some(U256::from(2000000000u64)));
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(3149));
}

#[test]
fn test_calculate_difficulty4() {
    let params = UnityEngineParams {
        rampup_upper_bound: U256::zero(),
        rampup_lower_bound: U256::zero(),
        rampup_start_value: U256::zero(),
        rampup_end_value: U256::zero(),
        lower_block_reward: U256::zero(),
        upper_block_reward: U256::zero(),
        difficulty_bound_divisor: U256::from(2048u64),
        difficulty_bound_divisor_unity: 20u64,
        minimum_difficulty: U256::from(16),
        block_time_lower_bound: 5u64,
        block_time_upper_bound: 15u64,
        block_time_unity: 10u64,
    };
    let calculator = DifficultyCalc::new(&params, Some(10u64), Some(U256::from(2000000000u64)));
    let mut parent_header = Header::default();
    parent_header.set_timestamp(1524528020u64);
    parent_header.set_difficulty(U256::from(16));
    parent_header.set_number(3);
    let mut grand_parent_header = Header::default();
    grand_parent_header.set_timestamp(1524528010u64);
    grand_parent_header.set_number(2);
    grand_parent_header.set_difficulty(U256::from(16));
    grand_parent_header.set_seal_type(SealType::PoS);
    let mut great_grand_parent_header = Header::default();
    great_grand_parent_header.set_timestamp(1524528005u64);
    great_grand_parent_header.set_number(1);
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(16));

    // Unity difficulty rule
    let calculator = DifficultyCalc::new(&params, Some(0u64), Some(U256::from(2000000000u64)));
    let difficulty = calculator.calculate_difficulty(
        &parent_header,
        Some(&grand_parent_header),
        Some(&great_grand_parent_header),
    );
    assert_eq!(difficulty, U256::from(17));
}