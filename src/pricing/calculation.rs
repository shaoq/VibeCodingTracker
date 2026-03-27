use super::cache::ModelPricing;

const TOKEN_THRESHOLD: i64 = 200_000;

/// Calculates total cost based on token usage and model pricing
///
/// The 200K threshold is based on total input context (input + cache_read + cache_creation).
/// When total input context exceeds 200K, above_200k prices apply to ALL token types,
/// matching Anthropic's pricing model where prompt length determines the pricing tier.
pub fn calculate_cost(
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_creation_tokens: i64,
    pricing: &ModelPricing,
) -> f64 {
    // The 200K threshold is about total input context per request
    let total_input_context = input_tokens + cache_read_tokens + cache_creation_tokens;
    let is_above_threshold = total_input_context > TOKEN_THRESHOLD;

    let input_price = if is_above_threshold {
        pricing.input_cost_per_token_above_200k_tokens
    } else {
        pricing.input_cost_per_token
    };
    let output_price = if is_above_threshold {
        pricing.output_cost_per_token_above_200k_tokens
    } else {
        pricing.output_cost_per_token
    };
    let cache_read_price = if is_above_threshold {
        pricing.cache_read_input_token_cost_above_200k_tokens
    } else {
        pricing.cache_read_input_token_cost
    };
    let cache_creation_price = if is_above_threshold {
        pricing.cache_creation_input_token_cost_above_200k_tokens
    } else {
        pricing.cache_creation_input_token_cost
    };

    let input_cost = input_tokens as f64 * input_price;
    let output_cost = output_tokens as f64 * output_price;
    let cache_read_cost = cache_read_tokens as f64 * cache_read_price;
    let cache_creation_cost = cache_creation_tokens as f64 * cache_creation_price;

    input_cost + output_cost + cache_read_cost + cache_creation_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost_below_threshold() {
        let pricing = ModelPricing {
            input_cost_per_token: 0.000001,
            output_cost_per_token: 0.000002,
            cache_read_input_token_cost: 0.0000001,
            cache_creation_input_token_cost: 0.0000005,
            input_cost_per_token_above_200k_tokens: 0.000002,
            output_cost_per_token_above_200k_tokens: 0.000004,
            cache_read_input_token_cost_above_200k_tokens: 0.0000002,
            cache_creation_input_token_cost_above_200k_tokens: 0.000001,
        };

        // Total input context = 1000 + 200 + 100 = 1300 < 200K → base prices
        let cost = calculate_cost(1000, 500, 200, 100, &pricing);
        let expected = 1000.0 * 0.000001 + 500.0 * 0.000002 + 200.0 * 0.0000001 + 100.0 * 0.0000005;
        assert_eq!(cost, expected);
    }

    #[test]
    fn test_calculate_cost_above_threshold() {
        let pricing = ModelPricing {
            input_cost_per_token: 0.000001,
            output_cost_per_token: 0.000002,
            cache_read_input_token_cost: 0.0000001,
            cache_creation_input_token_cost: 0.0000005,
            input_cost_per_token_above_200k_tokens: 0.000002,
            output_cost_per_token_above_200k_tokens: 0.000004,
            cache_read_input_token_cost_above_200k_tokens: 0.0000002,
            cache_creation_input_token_cost_above_200k_tokens: 0.000001,
        };

        // Total input context = 250K + 250K + 250K = 750K > 200K → above_200k prices for ALL
        let cost = calculate_cost(250_000, 250_000, 250_000, 250_000, &pricing);
        let expected = 250_000.0 * 0.000002   // input: above_200k
            + 250_000.0 * 0.000004            // output: above_200k (determined by input context)
            + 250_000.0 * 0.0000002           // cache_read: above_200k
            + 250_000.0 * 0.000001; // cache_creation: above_200k
        assert_eq!(cost, expected);
    }

    #[test]
    fn test_calculate_cost_context_threshold() {
        // The 200K threshold is about total input context, not individual token types
        let pricing = ModelPricing {
            input_cost_per_token: 0.000003,
            output_cost_per_token: 0.000015,
            cache_read_input_token_cost: 0.0000003,
            cache_creation_input_token_cost: 0.00000375,
            input_cost_per_token_above_200k_tokens: 0.000006,
            output_cost_per_token_above_200k_tokens: 0.0000225,
            cache_read_input_token_cost_above_200k_tokens: 0.0000006,
            cache_creation_input_token_cost_above_200k_tokens: 0.0000075,
        };

        // Case 1: No single type > 200K, but total input context > 200K
        // input=100K + cache_read=80K + cache_creation=30K = 210K > 200K → above_200k
        let cost1 = calculate_cost(100_000, 50_000, 80_000, 30_000, &pricing);
        let expected1 = 100_000.0 * 0.000006      // above_200k
            + 50_000.0 * 0.0000225                 // above_200k (output also affected)
            + 80_000.0 * 0.0000006                 // above_200k
            + 30_000.0 * 0.0000075; // above_200k
        assert_eq!(cost1, expected1);

        // Case 2: input=50K + cache_read=60K + cache_creation=40K = 150K < 200K → base
        // Even though total with output (50K+80K+60K+40K=230K) > 200K, output doesn't count
        let cost2 = calculate_cost(50_000, 80_000, 60_000, 40_000, &pricing);
        let expected2 = 50_000.0 * 0.000003        // base
            + 80_000.0 * 0.000015                  // base
            + 60_000.0 * 0.0000003                 // base
            + 40_000.0 * 0.00000375; // base
        assert_eq!(cost2, expected2);

        // Case 3: Large output but small context → base prices
        // input=50K + cache_read=0 + cache_creation=0 = 50K < 200K → base
        let cost3 = calculate_cost(50_000, 500_000, 0, 0, &pricing);
        let expected3 = 50_000.0 * 0.000003        // base
            + 500_000.0 * 0.000015                 // base (output doesn't affect threshold)
            + 0.0
            + 0.0;
        assert_eq!(cost3, expected3);
    }

    #[test]
    fn test_calculate_cost_exactly_200k() {
        let pricing = ModelPricing {
            input_cost_per_token: 0.000001,
            output_cost_per_token: 0.000002,
            cache_read_input_token_cost: 0.0000001,
            cache_creation_input_token_cost: 0.0000005,
            input_cost_per_token_above_200k_tokens: 0.000002,
            output_cost_per_token_above_200k_tokens: 0.000004,
            cache_read_input_token_cost_above_200k_tokens: 0.0000002,
            cache_creation_input_token_cost_above_200k_tokens: 0.000001,
        };

        // Total input context = 200K + 0 + 0 = 200K (not > 200K) → base price
        let cost_exact = calculate_cost(200_000, 50_000, 0, 0, &pricing);
        let expected = 200_000.0 * 0.000001 + 50_000.0 * 0.000002 + 0.0 + 0.0;
        assert_eq!(cost_exact, expected);

        // Total input context = 200_001 + 0 + 0 = 200_001 > 200K → above_200k price
        let cost_above = calculate_cost(200_001, 50_000, 0, 0, &pricing);
        let expected_above = 200_001.0 * 0.000002 + 50_000.0 * 0.000004 + 0.0 + 0.0;
        assert_eq!(cost_above, expected_above);

        // Boundary: split across types: 100K + 50K + 50_001 = 200_001 > 200K → above_200k
        let cost_split = calculate_cost(100_000, 30_000, 50_000, 50_001, &pricing);
        let expected_split =
            100_000.0 * 0.000002 + 30_000.0 * 0.000004 + 50_000.0 * 0.0000002 + 50_001.0 * 0.000001;
        assert_eq!(cost_split, expected_split);
    }

    #[test]
    fn test_calculate_cost_fallback_to_base() {
        let mut pricing = ModelPricing {
            input_cost_per_token: 0.000001,
            output_cost_per_token: 0.000002,
            cache_read_input_token_cost: 0.0000001,
            cache_creation_input_token_cost: 0.0000005,
            ..Default::default()
        };

        // Simulate normalization: fill above_200k with base prices
        pricing.input_cost_per_token_above_200k_tokens = pricing.input_cost_per_token;
        pricing.output_cost_per_token_above_200k_tokens = pricing.output_cost_per_token;
        pricing.cache_read_input_token_cost_above_200k_tokens = pricing.cache_read_input_token_cost;
        pricing.cache_creation_input_token_cost_above_200k_tokens =
            pricing.cache_creation_input_token_cost;

        // Total input context = 250K + 250K + 250K = 750K > 200K
        // But above_200k prices equal base prices, so cost is same as base
        let cost = calculate_cost(250_000, 250_000, 250_000, 250_000, &pricing);
        let expected = 250_000.0 * 0.000001
            + 250_000.0 * 0.000002
            + 250_000.0 * 0.0000001
            + 250_000.0 * 0.0000005;
        assert_eq!(cost, expected);
    }
}
