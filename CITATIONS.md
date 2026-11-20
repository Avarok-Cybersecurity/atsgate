# Citations

This file is the single source of truth for every source ATSGate cites. The README's
Citations section and the docs site's citations page are generated from the table below by
`cargo run -p xtask -- citations`, and CI fails when they drift (`-- citations --check`).

Every entry was opened before it was cited. Grades: **PR** peer-reviewed, **AX** arXiv
preprint (not peer-reviewed), **VD** vendor documentation, **IB** industry report or blog,
**CR** community project or repository, **LG** legal text or standard.

| id | title | authors | date | url | accessed | grade | supports |
|---|---|---|---|---|---|---|---|
| ats-screener-method | ATS Screener scoring methodology, dimensions and thresholds | S. Patel | 2026 | https://github.com/sunnypatell/ats-screener/tree/main/docs/src/content/docs/scoring | 2026-11-20 | CR | Baseline weights and strictness; its pass floors contradict each other between methodology.md and thresholds.md |
| ats-screener-research | ATS parsing, scoring and platform research notes | S. Patel | 2026-02 | https://github.com/sunnypatell/ats-screener/tree/main/research | 2026-11-20 | CR | Vendor behaviour caveats: Lever and Greenhouse do not auto-score or auto-reject |
| confit-v2 | ConFit v2: Improving Resume-Job Matching using Hypothetical Resume Embedding and Runner-Up Hard-Negative Mining | Yu, Xu, Xue, Zhang, Ma, Z. Yu | 2025-02 | https://arxiv.org/abs/2502.12361 | 2026-11-20 | AX | BM25 trails dense retrieval on full resume-to-job ranking |
| confit-v3 | ConFit v3: Improving Resume-Job Matching with LLM-based Re-Ranking | Yu et al. | 2026-05 | https://arxiv.org/abs/2605.09760 | 2026-11-20 | AX | State-of-the-art context; LLMs kept out of deterministic scoring |
| pjb | PJB: A Reasoning-Aware Benchmark for Person-Job Retrieval | Wang et al. | 2026-03 | https://arxiv.org/abs/2603.17386 | 2026-11-20 | AX | Domain variance exceeds model gains |
| workrb | WorkRB: A Community-Driven Evaluation Framework for AI in the Work Domain | De Lange et al. | 2026-03 | https://arxiv.org/abs/2604.13055 | 2026-11-20 | AX | Lexical baselines hold on skill normalization (BM25 47.4 MAP) but collapse on titles (2.5) |
| lcd-pjf | An Efficient Long-Context Ranking Architecture With Calibrated LLM Distillation: Application to Person-Job Fit | Jouanneau, Jouffroy, Palyart | 2026-01 | https://arxiv.org/abs/2601.10321 | 2026-11-20 | AX | Skill-fit scores and calibration |
| review-recruit | From Matching Models to Recruiting Agents: A Systematized Narrative Review of AI Recruitment Systems, Evaluation, and Governance | Zhao, Wei | 2026-09 | https://arxiv.org/abs/2609.04286 | 2026-11-20 | AX | Evidence-level evaluation and contestability |
| hybrid-emb | Semantic Candidate-Job Matching: A Comparative Evaluation of Dense Embedding Models in Hybrid Retrieval | Yashwant et al. | 2026-09 | https://arxiv.org/abs/2609.23307 | 2026-11-20 | AX | Hybrid lexical and dense retrieval |
| talentclef25 | Overview of the TalentCLEF 2025: Skill and Job Title Intelligence for Human Capital Management | Gasco et al. | 2025-07 | https://arxiv.org/abs/2507.13275 | 2026-11-20 | AX | Job-title and skill benchmark; training strategy beat model size |
| talentclef26 | Overview of the TalentCLEF 2026: Skill and Job Title Intelligence for Human Capital Management | Gasco et al. | 2026-06 | https://arxiv.org/abs/2606.31692 | 2026-11-20 | AX | Job-person matching task |
| nlpnorth | NLPnorth @ TalentCLEF 2025: Comparing Discriminative, Contrastive, and Prompt-Based Methods for Job Title and Skill Matching | Zhang, van der Goot | 2025-06 | https://arxiv.org/abs/2506.19058 | 2026-11-20 | AX | Method comparison on title and skill matching |
| esco-linking | Enhancing Job Matching: Occupation, Skill and Qualification Linking with the ESCO and EQF taxonomies | Saroglou et al. | 2025-12 | https://arxiv.org/abs/2512.03195 | 2026-11-20 | AX | Entity linking suits skills; sentence linking suits occupations |
| skillspan | SkillSpan: Hard and Soft Skill Extraction from English Job Postings | Zhang, Jensen, Sonniks, Plank | 2022 | https://arxiv.org/abs/2204.12811 | 2026-11-20 | PR | Span-annotated job-description dataset (NAACL 2022) |
| rethink-se | Rethinking Skill Extraction in the Job Market Domain using Large Language Models | Nguyen, Zhang, Montariol, Bosselut | 2024 | https://arxiv.org/abs/2402.03832 | 2026-11-20 | PR | In-context LLM extraction still trails supervised taggers (workshop paper) |
| decorte-xml | Extreme Multi-Label Skill Extraction Training using Large Language Models | Decorte et al. | 2023 | https://arxiv.org/abs/2307.10778 | 2026-11-20 | PR | Synthetic-data contrastive models beat literal-match supervision (workshop paper) |
| senger-survey | Deep Learning-based Computational Job Market Analysis: A Survey on Skill Extraction and Classification from Job Postings | Senger, Zhang, van der Goot, Plank | 2024 | https://arxiv.org/abs/2402.05617 | 2026-11-20 | PR | Catalogue of skill-extraction datasets (workshop paper) |
| resumebench | Beyond Human Labels: A Multi-Linguistic Auto-Generated Benchmark for Evaluating Large Language Models on Resume Parsing | Ling et al. | 2025-11 | https://aclanthology.org/2025.emnlp-main.1626 | 2026-11-20 | PR | Resume parsing benchmark (EMNLP 2025, synthetic) |
| layout-resume | Layout-Aware Parsing Meets Efficient LLMs: A Unified, Scalable Framework for Resume Information Extraction and Evaluation | Zhu et al. | 2025-10 | https://arxiv.org/abs/2510.09722 | 2026-11-20 | AX | Normalize layout before extracting resume fields |
| pdf-tools | A Comparative Study of PDF Parsing Tools Across Diverse Document Categories | Adhikari, Agarwal | 2024 | https://arxiv.org/abs/2410.09871 | 2026-11-20 | AX | Rule-based extractor choice (PyMuPDF and pdfium strongest) |
| tk-columns | Improving extraction from column resumes | Textkernel | n.d. | https://textkernel.com/learn-support/blog/improving-extraction-from-column-resumes | 2026-11-20 | VD | At least 15% of CVs use columns; naive stream order interleaves them |
| faire | FAIRE: Assessing Racial and Gender Bias in AI-Driven Resume Evaluations | Wen et al. | 2025-04 | https://arxiv.org/abs/2504.01420 | 2026-11-20 | AX | Name-based bias in LLM screeners varies by model |
| iso-bias | Evaluating Bias in LLMs for Job-Resume Matching: Gender, Race, and Education | Iso et al. | 2025-03 | https://arxiv.org/abs/2503.19182 | 2026-11-20 | PR | Education bias persists (NAACL 2025 Industry) |
| karvonen | Robustly Improving LLM Fairness in Realistic Settings via Interpretability | Karvonen, Marks | 2025-06 | https://arxiv.org/abs/2506.10922 | 2026-11-20 | AX | College names act as a demographic proxy despite anti-bias prompts |
| fairness-not-enough | Fairness Is Not Enough: Auditing Competence and Intersectional Bias in AI-powered Resume Screening | Webster | 2025-07 | https://arxiv.org/abs/2507.11548 | 2026-11-20 | AX | Audit competence as well as bias |
| popresume | PopResume: Causal Fairness Evaluation of LLM/VLM Resume Screeners with Population-Representative Dataset | Yu, Park, Moon | 2026-03 | https://arxiv.org/abs/2603.22714 | 2026-11-20 | AX | Causal fairness audit dataset |
| llm-hire-fairly | Can LLMs Hire Fairly? Racial Bias in Resume Screening | Gao, Jiang, Yan | 2026-06 | https://arxiv.org/abs/2606.28978 | 2026-11-20 | AX | Bias direction flips across model generations |
| bias-tails | Bias in the Tails: How Name-conditioned Evaluative Framing in Resume Summaries Destabilizes LLM-based Hiring | Nghiem et al. | 2026-04 | https://arxiv.org/abs/2604.19984 | 2026-11-20 | AX | Name-driven variation hides in score-distribution tails |
| audit-verdict | The Audit Decides the Verdict: Instrument Effects Rival Demographic Bias in LLM Decision Audits | Vohra, Ravikiran | 2026-09 | https://arxiv.org/abs/2609.09048 | 2026-11-20 | AX | Audit construction changes the finding (workshop paper) |
| task-audit | Hiring Discrimination and the Task Content of Jobs: Evidence from a Large-Scale Resume Audit | Braun et al. | 2026-04 | https://arxiv.org/abs/2604.01933 | 2026-11-20 | AX | Discretion widens gaps; objective rules narrow them |
| inj-measure | Measuring Real-World Prompt Injection Attacks in LLM-based Resume Screening | Zhang et al. | 2026-05 | https://arxiv.org/abs/2605.28999 | 2026-11-20 | AX | About 1% of real resumes carry hidden injections (USENIX Security 2026 listed) |
| resumeshield | ResumeShield: Channel Separation and an Open Benchmark for Indirect Prompt Injection in AI Resume Screening | Barach | 2026-07 | https://arxiv.org/abs/2609.20188 | 2026-11-20 | AX | Concealment is itself a detectable signal (precision 1.0, recall 0.944) |
| inj-multi | Prompt Injection in Automated Résumé Screening with Large Language Models: Single and Multi-Injection Settings | Baxi et al. | 2026-06 | https://arxiv.org/abs/2606.27287 | 2026-11-20 | AX | Manipulation dynamics across screeners (ACL Findings listed) |
| ai-sec-resume | AI Security Beyond Core Domains: Resume Screening as a Case Study of Adversarial Vulnerabilities in Specialized LLM Applications | Mu et al. | 2025-12 | https://arxiv.org/abs/2512.20164 | 2026-11-20 | AX | Invisible-keyword attacks |
| hidden-workers | Hidden Workers: Untapped Talent (Executive Summary) | Fuller, Raman, Sage-Gavin, Hines; Harvard Business School and Accenture | 2021-09 | https://pubwww.hbs.edu/managing-the-future-of-work/Documents/research/Hidden%20Workers%20Executive%20Summary.pdf | 2026-11-20 | IB | Recruiting systems exclude candidates for a gap in employment; ATSGate never penalizes gaps |
| kristal-gaps | Reducing discrimination against job seekers with and without employment gaps | Kristal, Nicks, Gloor, Hauser | 2022-12 | https://europepmc.org/article/PMC/PMC7614241 | 2026-11-20 | PR | Listing years worked instead of dates raised callbacks about 8% (vs no gap) and 15% (vs gap); Nature Human Behaviour 7(2):211-218 |
| gh-tm-faq | Talent Matching Data Processing FAQ | Greenhouse | n.d. | https://support.greenhouse.io/hc/en-us/articles/41131616864283 | 2026-11-20 | VD | No auto-reject; names and contact details excluded |
| gh-import | Why didn't the resume import correctly? | Greenhouse | n.d. | https://support.greenhouse.io/hc/en-us/articles/200721684 | 2026-11-20 | VD | Unrecognized characters break parsing |
| gh-types | Supported file types | Greenhouse | n.d. | https://support.greenhouse.io/hc/en-us/articles/360052218132 | 2026-11-20 | VD | Accepted input formats |
| wd-parse | Set Up Prospects and Candidates | Workday | n.d. | https://doc.workday.com/admin-guide/en-us/human-capital-management/recruiting/candidates/set-up-prospects-and-candidates/hdc1552497830785.html | 2026-11-20 | VD | Parse results vary with format and word order; avoid image resumes |
| taleo-attach | Attachments | Oracle | 22D | https://docs.oracle.com/en/cloud/saas/taleo-enterprise/22d/otrcg/c-attachment.html | 2026-11-20 | VD | Accepted attachment formats |
| icims-blog | What is CV/Resume parsing? | iCIMS | 2020-09 | https://www.icims.com/blog/what-is-cv-resume-parsing/ | 2026-11-20 | VD | Context only (vendor blog) |
| sap-kba | KBA 2951470 | SAP | n.d. | https://userapps.support.sap.com/sap/support/knowledge/en/2951470 | 2026-11-20 | VD | Resume field-mapping limits |
| onet-lic | O*NET Database License | U.S. Department of Labor, Employment and Training Administration | 31.0 | https://www.onetcenter.org/license_db.html | 2026-11-20 | LG | Embedding O*NET data is allowed under CC BY 4.0 with attribution |
| esco-dl | Download ESCO | European Commission | n.d. | https://esco.ec.europa.eu/en/use-esco/download | 2026-11-20 | LG | Bulk download sits behind a form, so ATSGate imports ESCO v1.2.1 through the public API instead; reuse terms are in esco-faq |
| esco-faq | ESCO frequently asked questions: reuse of ESCO | European Commission | n.d. | https://esco.ec.europa.eu/en/about-esco/faq | 2026-11-20 | LG | Free reuse for any purpose under Decision 2011/833/EU; services state "This service uses the ESCO classification of the European Commission"; modified versions must say so |
| onet-db | O*NET 31.0 Database | U.S. Department of Labor, Employment and Training Administration | 31.0 | https://www.onetcenter.org/database.html | 2026-11-20 | LG | Current release is 31.0, licensed CC BY 4.0; source of the embedded occupations, skills and technology examples |
| skillspan-data | SkillSpan data repository | Zhang et al. | 2023-08 | https://github.com/kris927b/SkillSpan | 2026-11-20 | CR | MIT-licensed test split (JSON, BIO skill and knowledge tags, house and tech subsets) used in the evaluation |
| techwolf-sentences | Synthetic-ESCO-skill-sentences | Decorte, Verlinden, Van Hautte, Deleu, Develder, Demeester (TechWolf) | 2023 | https://huggingface.co/datasets/TechWolf/Synthetic-ESCO-skill-sentences | 2026-11-20 | CR | 138,260 synthetic sentences, ten per ESCO v1.1.0 skill; CC BY 4.0 |
| talentclef25-corpus | TalentCLEF 2025 corpus: Skill and Job Title Intelligence for Human Capital Management | Gascó et al. (Avature) | 2025-04 | https://zenodo.org/records/15292308 | 2026-11-20 | CR | Task A job-title matching data (v0.5.0), CC BY 4.0; the evaluation uses its English validation split |
| hn-artcraft | ArtCraft: Senior Engineers, onsite San Francisco or Atlanta (Ask HN: Who is hiring? November 2026) | possibilistic | 2026-11-02 | https://news.ycombinator.com/item?id=50342041 | 2026-11-20 | IB | Real posting used as the generator's worked example |
| lightcast-faq | Lightcast Skill Taxonomy FAQs | Lightcast | n.d. | https://lightcast.io/our-data/taxonomies/open-skills/faqs | 2026-11-20 | VD | Use is set by the licence type requested; terms are not stated openly, so ATSGate does not embed it |
| schema-jp | JobPosting | schema.org | current | https://schema.org/JobPosting | 2026-11-20 | LG | Blueprint field alignment |
| jsonresume | resume-schema | JSON Resume | archived | https://github.com/jsonresume/resume-schema | 2026-11-20 | CR | Career SSOT base format (MIT) |
| ll144 | Automated Employment Decision Tools: Frequently Asked Questions | NYC Department of Consumer and Worker Protection | 2023 | https://www.nyc.gov/site/dca/businesses/faq-automated-employment-decision-tools.page | 2026-11-20 | LG | Bias audits compute selection or scoring rates and impact ratios across sex, race/ethnicity and intersectional categories |
| eu-aia-a3 | AI Act Annex III: High-Risk AI Systems | artificialintelligenceact.eu | 2024 | https://artificialintelligenceact.eu/annex/3/ | 2026-11-20 | LG | Recruitment AI is high-risk (Annex III 4(a)) |

## Verification log

The research report of 2026-11-20 flagged some sources as not opened. Each was re-opened on
2026-11-20 before it was cited here.

| id | outcome |
|---|---|
| hidden-workers | Verified against the primary HBS executive summary (the report had cited a secondary news article). Its text says most recruiting systems use "a gap in full-time employment" as a basis for exclusion. The full report PDF URL returned 404. |
| kristal-gaps | Verified. nature.com requires a login, so the abstract was read from the Europe PMC record of the open-access author manuscript (PMC7614241, DOI 10.1038/s41562-022-01485-6). |
| lightcast-faq | Verified at its new URL (the old one redirects). The page does not state the terms; it says use depends on the licence requested. |
| ll144 | Re-sourced. The DCWP FAQ PDF did not render as text, so the entry cites DCWP's HTML FAQ, which was opened and states the impact-ratio requirement. |
| lever-help | Dropped. Lever's help article returned an error page and its old copy returned 403; ATSGate cites no Lever document until one can be opened. |
| namingit-gaps | Dropped. The report cited it only through a search summary, and it is not needed for any ATSGate claim. |
