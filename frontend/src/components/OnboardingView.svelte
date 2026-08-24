// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

<script lang="ts">
  import { t, UI_LANGUAGES } from '../lib/i18n';
  import {
    buildOnboardingResult,
    clampGoalValue,
    nextOnboardingStep,
    ONBOARDING_STEPS,
    previousOnboardingStep,
    type OnboardingGoalType,
    type OnboardingResult,
  } from '../lib/onboarding';
  import type { LanguageCode } from '../lib/types/index';

  let {
    uiLang = 'en',
    onUiLangChange,
    onComplete,
    onSkip,
  }: {
    uiLang?: string;
    onUiLangChange: (lang: string) => void;
    onComplete: (result: OnboardingResult) => void;
    onSkip: () => void;
  } = $props();

  let step = $state(0);
  let selectedPracticeLanguage = $state<LanguageCode>('en');
  let goalType = $state<OnboardingGoalType>('time');
  let goalMinutes = $state(15);
  let goalWpm = $state(40);
  let goalAccuracy = $state(90);

  function pickUiLanguage(lang: string) {
    onUiLangChange(lang);
  }

  function next() {
    step = nextOnboardingStep(step);
  }

  function back() {
    step = previousOnboardingStep(step);
  }

  function finish() {
    onComplete(
      buildOnboardingResult(
        selectedPracticeLanguage,
        goalType,
        goalMinutes,
        goalWpm,
        goalAccuracy,
      ),
    );
  }
</script>

<div class="onboarding-overlay" role="dialog" aria-modal="true" aria-label={t(uiLang, 'onboarding.welcome_title')}>
  <div class="onboarding-card">
    <button class="skip-btn" onclick={onSkip}>{t(uiLang, 'onboarding.skip')}</button>

    <h1>{t(uiLang, 'onboarding.welcome_title')}</h1>
    <p class="subtitle">{t(uiLang, 'onboarding.subtitle')}</p>

    <div class="dots" aria-hidden="true">
      {#each Array(ONBOARDING_STEPS) as _, index}
        <span class="dot" class:active={index === step}></span>
      {/each}
    </div>

    {#if step === 0}
      <h2>{t(uiLang, 'onboarding.step_ui_lang')}</h2>
      <div class="lang-grid">
        {#each UI_LANGUAGES as [code, label] (code)}
          <button
            class="lang-option"
            class:selected={uiLang === code}
            onclick={() => pickUiLanguage(code)}
          >
            {label}
          </button>
        {/each}
      </div>
    {:else if step === 1}
      <h2>{t(uiLang, 'onboarding.step_practice_lang')}</h2>
      <div class="lang-grid">
        {#each UI_LANGUAGES as [code, label] (code)}
          <button
            class="lang-option"
            class:selected={selectedPracticeLanguage === code}
            onclick={() => (selectedPracticeLanguage = code as LanguageCode)}
          >
            {label}
          </button>
        {/each}
      </div>
    {:else}
      <h2>{t(uiLang, 'onboarding.step_goal')}</h2>
      <div class="goal-controls">
        <label for="onboarding-goal-type">{t(uiLang, 'settings.daily_goal_type')}</label>
        <select id="onboarding-goal-type" bind:value={goalType}>
          <option value="time">{t(uiLang, 'settings.goal_time')}</option>
          <option value="wpm">{t(uiLang, 'settings.goal_wpm')}</option>
          <option value="accuracy">{t(uiLang, 'settings.goal_accuracy')}</option>
        </select>
        {#if goalType === 'time'}
          <label for="onboarding-goal-minutes">{t(uiLang, 'onboarding.goal_minutes')}</label>
          <input
            id="onboarding-goal-minutes"
            type="number"
            min="0"
            max="1440"
            bind:value={goalMinutes}
            onchange={() => (goalMinutes = clampGoalValue('time', goalMinutes))}
          />
        {:else if goalType === 'wpm'}
          <label for="onboarding-goal-wpm">{t(uiLang, 'settings.daily_goal_wpm')}</label>
          <input
            id="onboarding-goal-wpm"
            type="number"
            min="0"
            max="300"
            bind:value={goalWpm}
            onchange={() => (goalWpm = clampGoalValue('wpm', goalWpm))}
          />
        {:else}
          <label for="onboarding-goal-accuracy">{t(uiLang, 'settings.daily_goal_accuracy')}</label>
          <input
            id="onboarding-goal-accuracy"
            type="number"
            min="0"
            max="100"
            bind:value={goalAccuracy}
            onchange={() => (goalAccuracy = clampGoalValue('accuracy', goalAccuracy))}
          />
        {/if}
      </div>
    {/if}

    <div class="nav-row">
      <button class="nav-btn secondary" onclick={back} disabled={step === 0}>
        {t(uiLang, 'onboarding.back')}
      </button>
      {#if step < ONBOARDING_STEPS - 1}
        <button class="nav-btn primary" onclick={next}>{t(uiLang, 'onboarding.next')}</button>
      {:else}
        <button class="nav-btn primary" onclick={finish}>
          {t(uiLang, 'onboarding.finish')}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .onboarding-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--bg) 88%, transparent);
  }
  .onboarding-card {
    position: relative;
    width: min(560px, calc(100vw - 3rem));
    max-height: calc(100vh - 4rem);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 2rem;
    background: var(--bg-sub);
    border: 1px solid var(--sub);
    border-radius: 10px;
  }
  h1 { font-size: 1.35rem; color: var(--main); margin: 0; }
  .subtitle { color: var(--sub); margin: 0; font-size: 0.9rem; }
  h2 { font-size: 0.95rem; color: var(--text); text-transform: uppercase; letter-spacing: 0.04em; margin: 0; }
  .skip-btn {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: none;
    border: none;
    color: var(--sub);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.8rem;
    text-decoration: underline;
  }
  .skip-btn:hover { color: var(--main); }
  .dots { display: flex; gap: 0.4rem; }
  .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--sub);
    opacity: 0.4;
  }
  .dot.active { opacity: 1; background: var(--main); }
  .lang-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: 0.5rem;
  }
  .lang-option {
    padding: 0.55rem 0.5rem;
    background: var(--bg);
    color: var(--text);
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.85rem;
    text-align: center;
  }
  .lang-option:hover { border-color: var(--sub); }
  .lang-option.selected {
    border-color: var(--main);
    color: var(--main);
    font-weight: bold;
  }
  .goal-controls {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-width: 16rem;
  }
  .goal-controls label {
    font-size: 0.75rem;
    color: var(--sub);
    text-transform: uppercase;
    margin-top: 0.35rem;
  }
  .goal-controls select,
  .goal-controls input {
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--sub);
    border-radius: 6px;
    padding: 0.45rem 0.6rem;
    font-family: inherit;
    font-size: 0.9rem;
  }
  .nav-row {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }
  .nav-btn {
    padding: 0.55rem 1.4rem;
    border-radius: 6px;
    border: 1px solid transparent;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.9rem;
  }
  .nav-btn.primary {
    background: var(--main);
    color: var(--bg);
  }
  .nav-btn.primary:hover { opacity: 0.85; }
  .nav-btn.secondary {
    background: transparent;
    color: var(--sub);
    border-color: var(--sub);
  }
  .nav-btn.secondary:hover:not(:disabled) { color: var(--main); border-color: var(--main); }
  .nav-btn:disabled { opacity: 0.4; cursor: default; }
</style>
