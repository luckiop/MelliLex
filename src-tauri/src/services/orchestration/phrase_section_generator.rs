use crate::errors::AppError;
use crate::models::{
    PhraseDefinitionData, PhraseSection1Overview, PhraseSection2Context, PhraseSection3Related,
};
use crate::services::orchestration::PhraseProgressiveEmitter;
use crate::services::phrase_service::PhraseService;
use log::info;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::timeout;

const RELATED_SECTION_TIMEOUT: Duration = Duration::from_secs(25);

fn empty_phrase_related() -> PhraseSection3Related {
    PhraseSection3Related {
        variations: Vec::new(),
        similar_phrases: Vec::new(),
        opposite_phrases: Vec::new(),
        see_also: Vec::new(),
        token_usage: None,
    }
}

/// Result type for parallel phrase section generation tasks.
pub enum PhraseSectionTaskResult {
    Section1(PhraseSection1Overview),
    Section2(PhraseSection2Context),
    Section3(PhraseSection3Related),
}

/// Handles parallel generation of all three progressive phrase sections.
pub struct PhraseSectionGenerator;

impl PhraseSectionGenerator {
    /// Spawns section 1/2/3 together and emits each as it completes.
    pub async fn generate_progressive<E: PhraseProgressiveEmitter + ?Sized>(
        phrase_service: Arc<PhraseService>,
        phrase: &str,
        language: &str,
        emitter: &E,
    ) -> Result<PhraseDefinitionData, AppError> {
        let mut section1_data: Option<PhraseSection1Overview> = None;
        let mut section2_data: Option<PhraseSection2Context> = None;
        let mut section3_data: Option<PhraseSection3Related> = None;

        let mut join_set: JoinSet<Result<PhraseSectionTaskResult, AppError>> = JoinSet::new();

        join_set.spawn({
            let svc = phrase_service.clone();
            let section_phrase = phrase.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating phrase section 1 (overview) [parallel] in {}",
                    section_language
                );
                svc.generate_section1_overview(&section_phrase, &section_language)
                    .await
                    .map(|(section, _)| PhraseSectionTaskResult::Section1(section))
                    .map_err(|e| {
                        log::error!("Phrase Section 1 error for '{}': {}", section_phrase, e);
                        AppError::from(e)
                    })
            }
        });

        join_set.spawn({
            let svc = phrase_service.clone();
            let section_phrase = phrase.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating phrase section 2 (context) [parallel] in {}",
                    section_language
                );
                svc.generate_section2_context(&section_phrase, &section_language)
                    .await
                    .map(|(section, _)| PhraseSectionTaskResult::Section2(section))
                    .map_err(|e| {
                        log::error!("Phrase Section 2 error for '{}': {}", section_phrase, e);
                        AppError::from(e)
                    })
            }
        });

        join_set.spawn({
            let svc = phrase_service.clone();
            let section_phrase = phrase.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating phrase section 3 (related) [parallel] in {}",
                    section_language
                );
                match timeout(
                    RELATED_SECTION_TIMEOUT,
                    svc.generate_section3_related(&section_phrase, &section_language),
                )
                .await
                {
                    Ok(Ok((section, _))) => Ok(PhraseSectionTaskResult::Section3(section)),
                    Ok(Err(e)) => {
                        log::warn!(
                            "Phrase section 3 error for '{}': {} — continuing without related phrases",
                            section_phrase,
                            e
                        );
                        Ok(PhraseSectionTaskResult::Section3(empty_phrase_related()))
                    }
                    Err(_) => {
                        log::warn!(
                            "Phrase section 3 timed out for '{}' after {}s — continuing without related phrases",
                            section_phrase,
                            RELATED_SECTION_TIMEOUT.as_secs()
                        );
                        Ok(PhraseSectionTaskResult::Section3(empty_phrase_related()))
                    }
                }
            }
        });

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(PhraseSectionTaskResult::Section1(section))) => {
                    info!("Emitting phrase section 1 (overview)");
                    emitter.emit_section1(&section)?;
                    section1_data = Some(section);
                }
                Ok(Ok(PhraseSectionTaskResult::Section2(section))) => {
                    info!("Emitting phrase section 2 (context)");
                    emitter.emit_section2(&section)?;
                    section2_data = Some(section);
                }
                Ok(Ok(PhraseSectionTaskResult::Section3(section))) => {
                    info!("Emitting phrase section 3 (related)");
                    emitter.emit_section3(&section)?;
                    section3_data = Some(section);
                }
                Ok(Err(e)) => {
                    log::error!("Phrase section generation error: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    log::error!("Phrase section task join error: {}", e);
                    return Err(AppError::from(anyhow::anyhow!("Task join error: {}", e)));
                }
            }
        }

        let section1 = section1_data.ok_or_else(|| {
            AppError::from(anyhow::anyhow!("Phrase section 1 was not generated"))
        })?;
        let section2 = section2_data.ok_or_else(|| {
            AppError::from(anyhow::anyhow!("Phrase section 2 was not generated"))
        })?;
        let section3 = section3_data.ok_or_else(|| {
            AppError::from(anyhow::anyhow!("Phrase section 3 was not generated"))
        })?;

        Ok(PhraseDefinitionData {
            section1,
            section2,
            section3,
        })
    }
}
