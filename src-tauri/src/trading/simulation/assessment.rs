use serde::Serialize;

/// Data/model eligibility is separate from a deterministic calculation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAssessmentView {
    pub model: String,
    pub performance_status: String,
    pub daily_context_bars: usize,
    pub intraday_bars: usize,
    pub timestamp_status: String,
    pub session_status: String,
    pub limitations: Vec<String>,
}

impl ReplayAssessmentView {
    pub fn lth(interval: &str, daily_context_bars: usize, intraday_bars: usize) -> Self {
        let daily = interval != "1m";
        Self {
            model: if daily {
                "dailyDiagnostic"
            } else {
                "intradaySample"
            }
            .into(),
            performance_status: if daily || intraday_bars == 0 {
                "notEvaluable"
            } else {
                "sampleOnly"
            }
            .into(),
            daily_context_bars,
            intraday_bars: if daily { 0 } else { intraday_bars },
            timestamp_status: "unverified".into(),
            session_status: "unverified".into(),
            limitations: if daily {
                vec![
                    "일봉 OHLC에서 장중 가격을 복원하거나 시가를 진입 시각의 가격으로 간주하지 않습니다.".into(),
                    "실제 분봉이 없어 장중 신호·체결·투자 성과를 평가할 수 없습니다.".into(),
                ]
            } else if intraday_bars == 0 {
                vec![
                    "유효한 분봉 관측이 없어 장중 신호·체결·투자 성과를 평가할 수 없습니다.".into(),
                ]
            } else {
                vec![
                    "최대 200개 분봉 조회는 장중 표본이며 3개월 전체 평가가 아닙니다.".into(),
                    "원본 시각의 시간대·DST·거래소 세션과 기간 전체 자료를 확인해야 합니다.".into(),
                    "현재 세션 계산은 고정 KST 근사이며 실제 장중 체결 경로를 재현하지 않습니다."
                        .into(),
                ]
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReplayAssessmentView;

    #[test]
    fn empty_minute_observations_are_not_evaluable() {
        let assessment = ReplayAssessmentView::lth("1m", 252, 0);
        assert_eq!(assessment.model, "intradaySample");
        assert_eq!(assessment.performance_status, "notEvaluable");
        assert_eq!(assessment.intraday_bars, 0);
        assert_eq!(assessment.daily_context_bars, 252);
        assert_eq!(
            ReplayAssessmentView::lth("1m", 252, 20).performance_status,
            "sampleOnly"
        );
    }
}
