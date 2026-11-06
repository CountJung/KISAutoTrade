//! Explicit unadjusted broker-chart price research. No provider/account/storage access.
pub(super) mod types;
mod validation;
use super::{
    replay, SectorDataSource, SectorDay, SectorPortfolioInput, SectorPrice, SECTOR_ENGINE_VERSION,
};
use sha2::{Digest, Sha256};
use types::*;

pub fn validate_broker_chart_input(input: &BrokerChartInput) -> Result<(), String> {
    validation::input(input)
}

/// Separate public contract: unknown corporate actions are acknowledged, never
/// represented as verified economic returns or promoted into the strict mode.
pub fn preview_broker_chart_price_approximation(
    input: &BrokerChartInput,
) -> Result<BrokerChartReport, String> {
    validate_broker_chart_input(input)?;
    let normalized = serde_json::to_vec(&serde_json::json!({
        "mode": "brokerChartPriceApproximation", "signalBasis": "unadjustedClose", "input": input,
    }))
    .map_err(|_| "INVALID_BROKER_CHART_JSON".to_string())?;
    let input_hash = hex::encode(Sha256::digest(normalized));
    let converted = accounting_input(input);
    // Strict preview is deliberately not called: its verification requirements
    // remain intact, and the internal source flags stay false throughout replay.
    let mut result = replay::replay_validated(&converted)?;
    let warnings = warnings(input);
    result.input_hash.clone_from(&input_hash);
    result.engine_version = format!("{SECTOR_ENGINE_VERSION}+broker-chart-price-v1");
    result.warnings.clone_from(&warnings);
    Ok(BrokerChartReport {
        mode: "brokerChartPriceApproximation".into(),
        signal_basis: "unadjustedClose".into(),
        dividend_return_included: false,
        economic_return_verified: false,
        input_hash,
        warnings,
        result,
    })
}

fn accounting_input(input: &BrokerChartInput) -> SectorPortfolioInput {
    let convert = |days: &[BrokerChartDay]| {
        days.iter()
            .map(|day| SectorDay {
                date: day.date.clone(),
                prices: day
                    .prices
                    .iter()
                    .map(|(symbol, price)| {
                        (
                            symbol.clone(),
                            SectorPrice {
                                // This field is the existing planner's signal slot, not an assertion
                                // of adjustment. The mode/source explicitly label it unadjustedClose.
                                adjusted_close: price.close,
                                open: price.open,
                                close: price.close,
                            },
                        )
                    })
                    .collect(),
            })
            .collect()
    };
    SectorPortfolioInput {
        initial_cash_usd: input.initial_cash_usd,
        cost_bps: input.cost_bps,
        source: SectorDataSource {
            name: input.source.name.clone(),
            retrieved_at: input.source.retrieved_at.clone(),
            synthetic: input.source.synthetic,
            adjustment_policy: "unadjustedSignalRawExecution".into(),
            calendar_source: match input.source.session_basis {
                BrokerChartSessionBasis::ObservedSpySessions => {
                    "observedSpySessions (not certified): "
                }
                BrokerChartSessionBasis::BrokerCalendar => "brokerCalendar (caller declaration): ",
            }
            .to_owned()
                + &input.source.calendar_evidence,
            coverage_verified: false,
            corporate_actions_verified: false,
        },
        history: convert(&input.history),
        evaluation: convert(&input.evaluation),
        expected_session_dates: input.expected_session_dates.clone(),
        corporate_actions: Vec::new(),
    }
}

fn warnings(input: &BrokerChartInput) -> Vec<String> {
    let mut warnings = vec![
        "brokerChartPriceApproximation 가격 근사 연구입니다. 비조정 종가를 신호·평가에 사용하며 배당 포함 총수익/경제적 수익을 검증하지 않았습니다.".into(),
        "비수정은 요청 플래그(KIS MODP=0 / Toss adjusted=false)를 뜻합니다. Toss 제공 시작 이전의 과거 구간은 false 요청에도 이미 수정되었을 수 있으며, 반환값이 경제적으로 완전한 원시가격임을 자동 인증하지 않습니다.".into(),
        "분할·배당 사건의 미확인을 명시적으로 허용한 가격 장부입니다. knownCorporateActions가 비어 있다는 것은 사건 부재 확인이 아닙니다. 알려진 사건이 하나라도 있으면 현재 미지원으로 전체 평가를 거부합니다.".into(),
        "coverageVerified와 corporateActionsVerified는 false로 유지합니다. 이 결과를 엄격 검증 모드로 자동 내보내거나 실제/기존 모의 원장과 합산하지 마세요.".into(),
        "모든 12종목과 선언 거래일의 일치는 검사했지만 공급원 진위·조정 의미·공식 거래일 완전성은 자동 인증하지 않았습니다. 거래일 공백이 있으면 실제 5/20거래일과 관측 5/20봉이 다를 수 있습니다.".into(),
        "신호·예산은 이전 완료 일봉만 사용하고 다음 거래일 시가에서 정수 수량을 계산합니다. nextOpen 사후 근사로 실제 09:50 표시가격·호가·유동성·체결을 복원하지 않습니다.".into(),
        "기본3×30%, 방어 같은3×15%/30%(SPY20일 두 신호), 공격2×47.5% 및 5%p 리밸런싱 규칙은 동일합니다. 초기 방어상태는 보통이며 기존 모의 보유/상태를 복원하지 않습니다.".into(),
        "독립 가상 USD 가격 장부의 비용·순손익입니다. 매도 후 순위 매수·정수주·현금 한도·차입/공매도 없음, 세금·현금이자0을 적용하며 주문/계좌/리스크 운영 경로를 호출하지 않습니다.".into(),
        "기말은 마지막 비조정 종가로 평가하고 청산비용을 추가 차감하지 않았습니다. USD f64 정밀도로 계산하며 매 거래 센트 반올림은 하지 않습니다.".into(),
        "회전율=(매수대금+매도대금)/초기자금×100이며 일 변동성은 표본 표준편차(모두 비연율화)입니다. 비용 제거 수익률은 같은 체결 경로에 비용을 더한 산술값입니다.".into(),
        "inputHash는 원본 파일 bytes가 아닌 고정 mode/signalBasis 태그와 전체 BrokerChartInput DTO(OHLCV·출처·확인·schemaVersion 포함)의 정규화 SHA256입니다. 중첩 result.inputHash도 같은 모드 입력을 가리킵니다.".into(),
        "짧은 회고 자료의 가격 근사로 전략 우월성·급락 방어·실거래 안전성을 확정할 수 없습니다.".into(),
    ];
    warnings.push(match input.source.session_basis {
        BrokerChartSessionBasis::ObservedSpySessions => "observedSpySessions: SPY 관측 날짜와 12종목을 맞춘 표본입니다. 공식 거래일 달력·전체 거래일 coverage를 확인했다는 뜻이 아닙니다.".into(),
        BrokerChartSessionBasis::BrokerCalendar => "brokerCalendar: calendarEvidence는 입력자 선언이며 공급원 달력을 프로그램이 별도로 인증하지 않았습니다.".into(),
    });
    if input.source.synthetic {
        warnings.insert(
            0,
            "합성 fixture 결과입니다. 실제 브로커 시세·실제 투자 성과가 아닙니다.".into(),
        );
    } else {
        warnings.insert(0, "실제 브로커 자료라는 입력자 선언을 보존합니다. 프로그램이 실제 자료나 경제적 수익임을 인증하지 않았습니다.".into());
    }
    if input.cost_bps == 20.0 {
        warnings.push("편도 총20bp 스트레스이며 기준10bp와 중복 가산하지 않습니다.".into());
    }
    warnings
}

#[cfg(test)]
mod tests;
