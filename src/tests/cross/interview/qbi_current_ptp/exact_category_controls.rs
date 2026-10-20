use super::*;

#[test]
fn current_ptp_keeps_fractional_source_cents_before_the_category_report() {
    for raw in [610_451, -10_451, 1, -1] {
        let mut page = ptp_page(0, true);
        page.ordinary = raw;
        let PartnerQbi::Statement { income, .. } = &mut page.qbi else { unreachable!() };
        *income = raw;
        let owner = page.record();
        let actual = owner
            .qbi
            .as_ref()
            .unwrap()
            .calculate(&owner, crate::tests::support::builders::k1_amount(0), 0, 239_250, FederalStatus::Single)
            .unwrap();
        // Exactly one half of the unchanged source cents, including odd
        // cents and both signs. No cent or Dollar has rounded in this API.
        assert_eq!(actual.ptp, crate::kernel::ratio::exact::ratio(raw, 2).unwrap());
        assert_eq!(actual.current.income, 0);
        assert_eq!(actual.historical, 0);
    }
    // The existing Dollar activity facade keeps its historical rounding and
    // unknown-classification priority while sharing the same policy factor.
    for (income, service, expected) in [
        (155, Some(true), 78),
        (-155, Some(true), -78),
        (155, None, 78),
        (-155, None, -155),
        (155, Some(false), 155),
    ] {
        let activity = crate::business_rules::QbiActivity { source_wages_cents: None, source_property_cents: None, source_income_cents: None,
            owner: None,
            income,
            wages: 0,
            property: 0,
            active: false,
            service,
            group: None,
        };
        let actual =
            crate::business_rules::eligible_qbi_activity(&activity, 239_250, FederalStatus::Single)
                .unwrap();
        assert_eq!(actual.income, expected);
        assert_eq!(actual.service, Some(false));
    }
}

#[test]
fn current_ptp_mixed_category_joins_raw_items_reit_and_annual_estate_outputs() {
    for sign in [1, -1] {
        let case = alloc::format!("ptp-mixed-category-{sign}");
        let mut draft = complete(FederalStatus::Single);
        // Ordinary PTP Schedule E reports +/-101; dividend ordinary reports1.
        // The independent fiduciary QBI statement is an apportioned source,
        // as in the existing TD9847 independent-DNI control, not another
        // entered taxable amount or a reconstructed activity/loss pool.
        draft.income[0] = wage(1, Owner::Taxpayer, (239_250 + 16_100 - sign * 101 - 1) * 100, 0);
        let wage_source = draft.income[0].clone();
        let mut session = Interview::new(TEST_USER, draft);
        ptp_save_prior_carry(&mut session, &alloc::format!("{case}-prior"), exchange);
        let dividends = Div1099 {
            more: DivMore {
                section199a: 25,
                ..Default::default()
            },
            answers: DivAnswers {
                reit_eligible: Some(true),
                after_death: None,
            },
            ..native_div1099("Synthetic qualifying REIT source".into(), 100, 0)
        };
        let dividend_request = save_topic(
            &session.draft,
            TaxSection::InterestDividends,
            vec![document(0, IncomeDocument::Div1099(dividends))],
        );
        accepted(&exchange(&mut session, dividend_request, &alloc::format!("{case}-reit"), false));
        let mut current = ptp_page(0, true);
        current.name = "Actual current SSTB source".into();
        current.ordinary = sign * 10_050;
        let PartnerQbi::Statement { income, .. } = &mut current.qbi else { unreachable!() };
        *income = sign * 10_050;
        let mut qualified = ptp_page(0, false);
        qualified.name = "Already qualified separate PTP source".into();
        let PartnerQbi::Statement { ptp_income, .. } = &mut qualified.qbi else { unreachable!() };
        *ptp_income = sign * 25;
        let mut estate = k1_1041("Independent fiduciary allocated PTP statement");
        estate.more.tax_exempt_interest = 50_000;
        estate.qbi = main_estate_qbi(sign * 150, 0);
        estate.qbi.statements[0].kind = crate::EstateQbiKind::PubliclyTradedPartnership;
        estate.qbi.statements[0].name = "Actual apportioned non-SSTB PTP statement".into();
        let sources = vec![
            document(0, IncomeDocument::K1Partnership(current.clone())),
            document(0, IncomeDocument::K1Partnership(qualified.clone())),
            document(0, IncomeDocument::K1EstateTrust(estate.clone())),
        ];
        let request = save_topic(&session.draft, TaxSection::PassThrough, sources);
        let display = ptp_estimate_and_save(&mut session, request, &case, exchange, credit_estimate_documents_from_save);
        let result = supported(&session.draft);
        // Entity annual output is already +/-2 Dollars, explicitly lifted to
        // +/-200 cents WITHOUT reconstructing its150 source cents or phasing.
        // Positive:5025+25+25+200=5275c ->53; negative:-5025-25+25-200=-5225c ->-52.
        let expected_category = if sign > 0 { 53 } else { -52 };
        if let Ok(root) = std::env::var("QBI_CURRENT_PTP_CASE_DIR") {
            let directory = std::path::Path::new(&root).join(alloc::format!("{case}-oracle"));
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join("financial-oracle.json"), serde_json::to_vec_pretty(
                &serde_json::json!({"pid":std::process::id(),"current_source_cents":sign*10_050,
                    "exact_current_eligible_cents":sign*5_025,"already_qualified_raw_ptp_cents":sign*25,
                    "qualified_reit_source_cents":25,"estate_source_cents":sign*150,
                    "estate_existing_annual_output_dollars":sign*2,
                    "actual_category_dollars":value(&result,"qbi_reit_ptp"),
                    "expected_category_dollars":expected_category,
                    "actual_carry_dollars":value(&result,"reit_ptp_loss_carryforward"),
                    "expected_carry_dollars":3000-expected_category,
                    "scope":"Neutral actual exchanges precede independent financial assertions; existing annual estate output is preserved, not a new source-cent estate engine"})
            ).unwrap()).unwrap();
        }
        assert_eq!(session.draft.income[0], wage_source);
        ptp_reopen_and_cold_resave(
            &mut session,
            &[
            IncomeDocument::K1Partnership(current),
            IncomeDocument::K1Partnership(qualified),
            IncomeDocument::K1EstateTrust(estate),
        ],
            &result,
            &case,
            exchange,
        );
        assert_eq!(display, crate::interview::estimate_text(&result));
        assert_eq!(value(&result, "agi"), 239_250 + 16_100);
        assert_eq!(value(&result, "qbi_taxable_income"), 239_250);
        assert_eq!(value(&result, "qbi_reit_ptp"), expected_category);
        assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 3000 - expected_category);
        assert_eq!(value(&result, "qbi_deduction"), 0);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    }
}

#[test]
fn current_ptp_signed_sources_reach_actual_joint_injured_owner_columns() {
    // Same actual source facts/allocation API as the existing Form8379
    // fixture. Joint midpoint478500: W2s506700 +REIT4000 +PTP101-101
    // =AGI510700; standard32200 ->preQBI478500. Taxpayer and Spouse have
    // separate actual qualified REIT2002/1998 and signed PTP+10050c/-10050c.
    let draft = crate::tests::attribution::injured_spouse::injured_return(
        (486_700, 100_000),
        (20_000, 10_000),
        0,
        crate::tests::attribution::injured_spouse::allocation(),
    );
    let mut session = Interview::new(TEST_USER, draft);
    let mut dividends = Vec::new();
    for (owner, amount) in [(Owner::Taxpayer, 200_200), (Owner::Spouse, 199_800)] {
        let page = Div1099 {
            more: DivMore {
                section199a: amount,
                ..Default::default()
            },
            answers: DivAnswers {
                reit_eligible: Some(true),
                after_death: None,
            },
            ..native_div1099(alloc::format!("Actual {owner:?} qualifying REIT"), amount, 0)
        };
        let mut item = document(0, IncomeDocument::Div1099(page));
        item.owner = owner;
        dividends.push(item);
    }
    let request = save_topic(&session.draft, TaxSection::InterestDividends, dividends);
    accepted(&exchange(&mut session, request, "ptp-injured-real-reit-save", false));
    let mut sources = Vec::new();
    for (owner, raw) in [(Owner::Taxpayer, 10_050), (Owner::Spouse, -10_050)] {
        let mut page = ptp_page(0, true);
        page.name = alloc::format!("Actual {owner:?} signed current SSTB PTP");
        page.ordinary = raw;
        let PartnerQbi::Statement { income, .. } = &mut page.qbi else { unreachable!() };
        *income = raw;
        let mut item = document(0, IncomeDocument::K1Partnership(page));
        item.owner = owner;
        sources.push(item);
    }
    let request = save_topic(&session.draft, TaxSection::PassThrough, sources);
    let estimated = exchange(
        &mut session,
        credit_estimate_documents_from_save(&request),
        "ptp-injured-real-sources-estimate",
        true,
    );
    let Reply::Response(P::IncomeRunningEstimate { estimate: display, .. }) = estimated else {
        panic!("actual injured candidate estimate")
    };
    accepted(&exchange(&mut session, request, "ptp-injured-real-sources-save", false));
    let result = supported(&session.draft);
    // Exact currentPTP sources +/-5025c cancel in the joint category.
    // Actual owner cents205225/194775 ->joint4000, TP2052, SP1948.
    // Joint deduction800: actual TP2052/4000*800=410.4 ->410;
    // SP gets390 residual. Early per-document DollarPTP would2053/1947
    // and411/389, so these actual emitted owner columns distinguish the bug.
    if let Ok(root) = std::env::var("QBI_CURRENT_PTP_CASE_DIR") {
        let directory = std::path::Path::new(&root).join("ptp-injured-real-sources-oracle");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("financial-oracle.json"),serde_json::to_vec_pretty(
            &serde_json::json!({"pid":std::process::id(),"actual_category":value(&result,"qbi_reit_ptp"),
                "expected_category":4000,"actual_qbi_deduction":value(&result,"qbi_deduction"),
                "expected_qbi_deduction":800,
                "actual_taxpayer_qbi_column":value(&result,"form_8379_supplemental_qbi_b"),
                "actual_spouse_qbi_column":value(&result,"form_8379_supplemental_qbi_c"),
                "expected_taxpayer_qbi_column":410,"expected_spouse_qbi_column":390,
                "actual_source_owner_cent_pools":[205225,194775],
                "scope":"Actual normalSave and Estimate plus fullstate exports precede independent emittedForm8379 owner financial assertions; no direct kernel-only proof"})
        ).unwrap()).unwrap();
    }
    let saved = session.draft.clone();
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    assert_eq!(page_documents(&reopened).len(), 2);
    assert_eq!(page_documents(&reopened)[0].owner, Owner::Taxpayer);
    assert_eq!(page_documents(&reopened)[1].owner, Owner::Spouse);
    for item in page_documents(&reopened) {
        assert!(item.id > 0);
    }
    accepted(&exchange(&mut session, reopened, "ptp-injured-real-sources-resave", false));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&exchange(&mut cold, reopened, "ptp-injured-real-sources-cold-resave", false));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), result);
    assert_eq!(display, crate::interview::estimate_text(&result));
    assert_eq!(value(&result, "agi"), 510_700);
    assert_eq!(value(&result, "qbi_taxable_income"), 478_500);
    assert_eq!(value(&result, "qbi_reit_ptp"), 4000);
    assert_eq!(value(&result, "qbi_deduction"), 800);
    assert_eq!(value(&result, "form_8379_supplemental_qbi_a"), 800);
    assert_eq!(value(&result, "form_8379_supplemental_qbi_b"), 410);
    assert_eq!(value(&result, "form_8379_supplemental_qbi_c"), 390);
    assert!(
        !result.review_notes.iter().any(|n| n.display.key == "WARN_8379_QBI_OWNER_UNAVAILABLE")
    );
}

#[test]
fn current_ptp_canonical_owner_boundary_refuses_nonfooting_source_provenance() {
    use crate::kernel::ratio::exact;
    // An independently reported joint4000 cannot be assigned from only
    // actual owner sources2000/1999. Missing1 stays unavailable.
    assert!(
        exact::project_rounded_pair(
            4000,
            &exact::whole(400_000),
            [exact::whole(200_000), exact::whole(199_900)],
            0
        )
        .is_none()
    );
    // Missing2 known source cents are MASKED by the same4000 joint-Dollar
    // report, yet cross the actual TP owner boundary205249 ->205251 cents.
    // The two actual owner amounts must conserve400002 exactjointcents;
    // a rounded-only footing check would silently return2052/1948 instead.
    assert!(
        exact::project_rounded_pair(
            4000,
            &exact::whole(400_002),
            [exact::whole(205_249), exact::whole(194_751)],
            0
        )
        .is_none()
    );
    assert_eq!(
        exact::project_rounded_pair(
            4000,
            &exact::whole(400_002),
            [exact::whole(205_251), exact::whole(194_751)],
            0
        ),
        Some([2053, 1947])
    );
    // A correct exact source sum still cannot certify an independently wrong
    // reported joint. Nor may an invalid first-owner selector be guessed.
    assert!(
        exact::project_rounded_pair(
            4001,
            &exact::whole(400_002),
            [exact::whole(205_251), exact::whole(194_751)],
            0
        )
        .is_none()
    );
    assert!(
        exact::project_rounded_pair(
            0,
            &exact::whole(0),
            [exact::ratio(1, 2).unwrap(), exact::ratio(-1, 2).unwrap()],
            2
        )
        .is_none()
    );
}
