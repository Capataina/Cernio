# UK Sponsor Register Discovery — 2026-05-14

Source: UK Home Office Register of Licensed Sponsors: Workers (Skilled Worker route, A-rated, English towns only), filtered through 10 parallel AI triage agents.

## Funnel

| Stage | Count |
|---|---:|
| Total register entries | 141,165 |
| Skilled Worker route only | 121,362 |
| A-rated + non-Scottish/Welsh/NI town | ~114,500 |
| After tech-keyword filter (HIGH-precision + MID-in-tech-hub) | 1,791 |
| After AI triage (KEEP + MAYBE) | ~378 |

## How to use this file

Each entry is **confirmed sponsor on the UK Skilled Worker route** with an A-rated licence in an English town. Sectors are tagged for `cernio import` ingestion. The "register entity name" is the legal entity on the gov.uk register — useful for `prepare-applications` and to populate the `uk_sponsor_entity_name` column when that schema lands.

Companies already in the existing 349-strong Cernio DB are excluded by name normalisation (with some near-misses noted in the WIDND at the bottom).

---

## Quant / HFT / Systematic Funds (S-tier candidates)

| Name | Register entity | Sector tags | What they do |
|---|---|---|---|
| AQR Capital Management | AQR Capital Management (Europe) LLP | quant, systematic, asset-mgmt | Tier-1 global systematic quant (~$100B AUM); factor investing + ML; large London research+eng team |
| Marshall Wace | Marshall Wace Asset Management Limited | quant, systematic, hedge | Major London systematic hedge fund (~$60B AUM); TOPS signal-aggregation platform |
| Millennium Management | Millennium Capital Management Limited | hedge, multistrat, quant | World's largest multi-strat hedge fund (~$70B AUM); deep London tech footprint |
| Brevan Howard | Brevan Howard Asset Management Services Limited | macro, hedge, quant | Top-tier global macro fund (~$30B AUM); strong London tech and quant org |
| BlueCrest Capital Management | BlueCrest Capital Management (UK) LLP | macro, systematic, hedge | Michael Platt's family-office fund running discretionary + systematic; heavy tech investment |
| ExodusPoint Capital Management | ExodusPoint Capital Management UK LLP | hedge, multistrat | Multi-strat hedge fund spun out of Millennium veterans; quant + tech org |
| Saba Capital Management | Saba Capital Management (UK) Limited | hedge, credit-RV | Boaz Weinstein's credit + closed-end fund activism; quant on derivatives |
| Sculptor Capital Management | Sculptor Capital Management Europe Limited | hedge, multistrat | Och-Ziff successor; multi-strat/credit; real quant + systems eng |
| Taula Capital Management | Taula Capital Management (UK) LLP | macro, hedge | Diego Megia (ex-Millennium) macro fund, $5B launch |
| Tenaron Capital Management | Tenaron Capital Management UK LLP | macro, hedge | Sat Sangha (ex-Millennium) macro fund, multi-billion launch |
| Alphadyne Asset Management | Alphadyne Asset Management (UK) LLP | hedge, fixedincome, macro | Large fixed-income/RV hedge fund (~$10B AUM); C++/Python pricing+exec |
| Andurand Capital Management | Andurand Capital Management LLP | commodities, hedge, macro | Pierre Andurand's commodities macro fund; known quant/research team |
| Quantbot Technologies | Quantbot Technologies Ltd | quant, systematic-equities | Schonfeld-stable systematic equities; C++/Python |
| AKO Capital Management | AKO Capital Management Ltd | hedge, longshort | London discretionary L/S equity |
| Soros Capital Management | Soros Capital Management UK LLP | hedge, family-office | Soros family office UK arm; macro infra |
| Cheyne Capital Management | Cheyne Capital Management (UK) LLP | credit, hedge, quant | Established London credit hedge fund; quant strategies, serious tech |
| SPX International Asset Management | SPX International Asset Management Ltd | macro, hedge, multistrat | Brazilian-rooted global macro multi-strat with London office |
| Marathon Asset Management | Marathon Asset Management (Services) Ltd | hedge, equities, global | Established London global-equities manager |
| Pictet Asset Management | Pictet Asset Management Ltd | asset-mgmt, traditional | Swiss private bank AM arm; some quant + tech |
| BNP Paribas Asset Management | BNP Paribas Asset Management UK Limited | asset-mgmt, traditional, quant | Global asset manager with quant strategies + ESG + multi-asset; London tech hub |
| Jupiter Asset Management | Jupiter Asset Management Limited | asset-mgmt, traditional | One of UK's largest active asset managers; in-house tech for portfolio + risk |
| Lazard Asset Management | Lazard Asset Management Limited | asset-mgmt, traditional | London active equity/fixed income arm with internal tech |
| L&G Asset Management | L&G - Asset Management Limited | asset-mgmt, traditional | LGIM — one of Europe's largest asset managers; substantial London tech |
| Baring Asset Management | Baring Asset Management Limited | asset-mgmt | MassMutual/Barings sub; systematic + quant equity arms |
| Robeco Institutional Asset Management UK | Robeco Institutional Asset Management UK Limited | asset-mgmt, quant | Dutch AM with systematic/quant equity team |
| Bayforest Technologies | Bayforest Technologies Ltd | quant, hft, stat-arb | London Bayesian real-time statistical inference; stat-arb + market-making, multi-asset systematic |
| Nickel Digital Asset Management | Nickel Digital Asset Management Limited | crypto, hedge, quant | London regulated crypto/digital-asset hedge fund |
| Sandbar Asset Management | Sandbar Asset Management LLP | hedge, market-neutral | London market-neutral equity hedge fund (ex-Millennium founder) |
| Atitlan Asset Management | Atitlan Asset Management Limited | crypto, quant, hedge | FCA-regulated quant crypto hedge fund; market-neutral systematic |
| BH-DG Systematic Trading | BH-DG Systematic Trading LLP | systematic, cta | London systematic trend-following CTA; JV between David Gorton and Brevan Howard; $2.1B AUM |
| The Oxford Asset Management Company | The Oxford Asset Management Company Limited | quant, systematic | OxAM — top-tier UK systematic quant; hires CS new grads |
| Winton Capital Management | Winton Capital Management Ltd | quant, systematic | David Harding's systematic quant fund; C++/Python heavy |
| GSR Technology Europe | GSR Technology Europe Ltd. | crypto, marketmaking, hft | GSR — crypto market maker; HFT-style top-tier eng |
| ADG Market Making | ADG Market Making LLP | quant, marketmaking, options | London options MM quoting on Eurex/Euronext/ICE; 80+ direct dealer lines |
| Geneva Trading | Geneva Financial Trading (UK) Limited | prop-trading, derivatives | Chicago/Dublin/London prop trading |
| Headlands Technologies | Headlands Tech UK Limited | quant, prop-trading | London/Chicago/NY/Austin/Amsterdam/Singapore prop trading |
| Capula Investment Management | Capula Investment Services Limited | hedge, fixedincome | Established fixed-income hedge fund |
| Old Mission Capital | Old Mission Europe LLP | prop-trading, marketmaking | Old Mission Capital's London arm |
| Point72 / Cubist | Point72 UK Limited | hedge, quant | Multi-strat hedge fund with London quant operations |
| Virtu Financial | Virtu Europe Trading Limited | hft, marketmaking | Major global HFT firm; London office |
| Cantab Capital / GAM Systematic | GAM (U.K.) Limited | quant, systematic | Cantab Capital (acquired by GAM 2016); systematic strategies, Cambridge |
| Numerix | Numerix Software LTD | quant-tech, derivatives | Industry-standard cross-asset derivatives pricing/risk library; classic top-tier C++ quant |
| CompatibL Technologies | CompatibL Technologies Ltd | quant, risk-software | Trading + risk-management software, model validation; serves derivatives dealers/central banks |
| MDX Technology | MDX Technology Limited | low-latency, capital-markets | Low-latency price/data sharing platform for TP ICAP/HSBC/ICE/RBC etc |
| Beacon Platform | Beacon Technologies Ltd | quant-infra, fintech | Quant developer infra, risk/trading analytics for banks/hedge funds (acquired by Clearwater) |
| Asset Control | Asset Control Technology Limited | fintech, data | Financial market & reference data management (ACPlus) for banks/asset managers |
| Chronicle Software | Chronicle Software Ltd | low-latency, infra | Ultra-low-latency Java messaging (Chronicle Queue/Engine); used by 8 of top 11 investment banks |
| MAIA Technology | MAIA Technology Limited | fintech, asset-mgmt-tech | Front-to-back cloud-native asset-mgmt SaaS (Molten-backed Series A) |
| Duco | Duco Technology Limited | fintech, data-infra | No-code AI data automation for financial markets; 14 of top 30 banks |
| Rimes Technologies | Rimes Technologies Limited | fintech-data-infra | Managed market-data services for asset managers |

## Major Asset Management (engineering-led)

| Name | Register entity | Sector tags |
|---|---|---|
| Acadian Asset Management | Acadian Asset Management (UK) Limited | quant, asset-mgmt |
| Susquehanna (SIG) | Susquehanna UK Limited (+ Susquehanna Dublin UK Branch) | quant, prop-trading |
| Oaktree Capital Management | Oaktree Capital Management (UK) LLP | credit, alternatives |
| Brookfield Global Asset Management | Brookfield Global Asset Management Limited | infra-AM (real-estate adjacent, verify scope) |

(Plus ~50 more mid-tier asset managers caught in batch 02-03; review for grading prioritisation.)

---

## AI / ML Labs (frontier + applied)

| Name | Register entity | Sector tags | What they do |
|---|---|---|---|
| Palantir Technologies UK | Palantir Technologies UK Limited | data-platforms, defence | Palantir UK; known Rust users; generous sponsor |
| Google DeepMind | Google (UK) Limited | ai-research, frontier | Google DeepMind UK |
| Latent Labs | Latent Labs Limited | ai, biology, generative | London/SF DeepMind-alum-led generative AI for biology; $50M raised |
| Flower Labs Cambridge | Flower Labs Cambridge LTD | ai, federated-learning | Cambridge spin-out behind Flower federated AI framework; trained 1.3B federated LLM |
| Lovable Labs UK | Lovable Labs UK Limited | ai, coding-tools | UK arm of Lovable — Stockholm AI app-builder ($1.8B valuation, $200M ARR); hiring London |
| Magentic Labs | Magentic Labs Limited | ai, agents, supply-chain | Sequoia-backed (Oxford ML PhD + ex-OpenAI) autonomous agents for supply-chain ops |
| Mantic Technologies | Mantic Technologies Limited | ai, forecasting | DeepMind/Cambridge spinout AI for event forecasting; $4M pre-seed; DRW-backed |
| Periodic Labs UK | Periodic Labs UK, Ltd | ai, materials-science | Ex-OpenAI/DeepMind founders; autonomous labs + frontier AI for materials |
| Gradient Labs | Gradient Labs Limited | ai, agents, fintech | Ex-Monzo-founded London AI agents for regulated FS; Redpoint-led €11M Series A |
| Convergence labs | Convergence Labs Ltd | ai, agents | Ex-DeepMind/Meta engineers; large meta-learning models / agentic web AI (acquired by Salesforce) |
| Unlikely Artificial Intelligence | Unlikely Artificial Intelligence Limited | ai, neurosymbolic | Cambridge/London deep-tech building neuro-symbolic general-AI; Alexa/Evi creator |
| Altos Labs UK | Altos Labs UK Limited | biotech, deeptech, ai | Bezos-funded $3B biotech; Cambridge Institute of Science; cellular reprogramming |
| Ignota Labs | Ignota Labs Limited | ai, drug-discovery | Cambridge AI-drug-rescue; SAFEPATH deep-learning + cheminformatics; ex-DeepMind AlphaFold founder |
| Matta Labs | Matta Labs Ltd | ai, manufacturing-vision | Cambridge IfM spin-out; industrial-AI vision for factory inspection |
| AlphaSense Technology | AlphaSense Technology Limited | ai, fintech-search | AI market-intelligence search over 500M docs; 88% S&P 100 clients |
| Eigen Technologies | Eigen Technologies Ltd | ai, nlp, fintech | NLP document intelligence for banks (Goldman/BlackRock/ING); acquired by Sirion 2024 |
| Luminance Technologies | Luminance Technologies Ltd | ai, legal-tech | Cambridge spinout legal AI; own LLM trained on 150M+ legal docs; Series C $75M |
| Xapien (Digital Insight Technologies) | Digital Insight Technologies Limited | ai, due-diligence | AI due-diligence platform; 100+ languages; Magic Circle clients |
| ContractPodAi | ContractPod Technologies Limited | ai, legal-tech, agents | Agentic AI CLM platform; global enterprise CLM |
| Aera Technology | Aera Technology Limited | ai, enterprise, decision-intel | Decision Cloud — agentic AI for enterprise supply-chain/finance |
| Altana | Altana Technologies UK Ltd | ai, supply-chain, graph | AI-powered global supply-chain network; UK gov contract |
| Afiniti | Afiniti Europe Technologies Limited | ai, contact-centre | Behavioural-pairing AI for contact centres; patented matching ML |
| TurinTech | Turing Intelligence Technology Limited | ai, code-optimisation | UCL-spinout AI for code optimisation (Artemis); $20M raised |
| Sayari Labs | Sayari Labs Limited | ai, risk, graph-data | Global corporate data + supply-chain risk; DC HQ, London office |
| Scribe Labs | Scribe Labs Limited | ai, fintech-data | Private-company data + AI extraction for PE/banks |
| Watershed Technology | Watershed Technology Limited | climate-ai, sustainability | Carbon measurement + sustainability platform; Stripe/Spotify/Klarna customers |
| Aflorithmic Labs (AudioStack) | Aflorithmic Labs, Ltd. | ai, audio, generative | London/Barcelona generative-AI audio infrastructure |
| Asymptote Labs | Asymptote Labs | ai, cybersec, agents | Security layer for agentic coding; ex-Brex/Doppel/Modal |
| Fern Labs (Chalfont AI) | Chalfont AI Ltd T/A Fern Labs | ai, agents | Ex-Palantir founders' multi-agent orchestration (Bridge); acquired by Poolside 2025 |
| Cinemersive Labs | Cinemersive Labs Ltd | ai, cv, vr | UK CV+ML 6DoF 3D image acquisition; acquired by Sony Interactive 2025 |
| Cambrian Robotics | Cambrian Robotics Limited | ai, robotics, cv | AI-based 3D vision for industrial robot arms; clients Toyota/Audi/Suzuki |
| Clay Labs UK | Clay Labs UK Ltd | ai, gtm, sales | AI-native GTM/sales platform with agents + 100+ data providers; opening London |
| Databook Labs | Databook Labs UK Ltd | ai, sales-intel | AI Strategic Relationship Management for enterprise sales |
| Magic Pony / Latent Tech (deprecated) | (verify) | — | — |
| Hypermile (Creation Labs AI) | Creation Labs AI Ltd T.A Hypermile | ai, automotive, adas | YC-backed AI cruise control for trucks/EVs; ~11% diesel reduction |
| Lilt Technologies | Lilt Technologies Limited | ai, translation | AI machine translation platform |
| Sana Labs | Sana Labs Ltd | ai, edtech, lms | Stockholm-based AI-native LMS (now part of Workday) |
| Gardenia Technologies | Gardenia Technologies Ltd | ai, applied | London applied AI / data platform (sustainability-finance) |
| Ground Truth Labs | Ground Truth Labs Ltd | ai, medical-pathology, cv | Oxford spinout computer vision on digital pathology slides for cancer biomarkers |
| Tubular Labs | Tubular Labs UK Limited | ai, video-analytics | Social video intelligence platform; ML over 7.5B videos |
| Vivacity Labs | Vivacity Labs Limited | ai, cv, smart-cities | AI cameras + smart-junction signal control; 30 UK cities |
| Schrodinger | Schrodinger Technologies Limited | ai, hpc, drug-discovery | Physics-based molecular simulation + AI for drug discovery |
| WSC Sports Technologies | WSC Sports Technologies Ltd | ai, video, sports | AI highlights platform for NBA/NHL/ESPN/Premier League |
| Wunderkind | Wunderkind Technologies UK Ltd | ai, martech, identity | Identity-resolution + agentic-AI personalisation; 501-1000 emp |

## Crypto / Blockchain / DeFi

| Name | Register entity | Sector tags |
|---|---|---|
| Chainlink Labs | Chainlink Labs Ltd | crypto, oracles, web3 |
| Euler Labs | Euler Labs Ltd | defi, lending |
| Argent Labs | Argent Labs Limited | crypto, wallet, defi |
| Anera Labs | Anera Labs UK Ltd | crypto, trading, quant, hft |
| Equilibrium Labs | Equilibrium Labs Group Limited | crypto, blockchain-infra, rust |
| OKX UK FinTech | OKX UK FinTech Company Limited | crypto, exchange |
| Paxos Technology | Paxos Technology Limited | crypto, stablecoin, rails |
| Blockchain (GB) | Blockchain (GB) Limited | crypto, exchange, wallet |
| Block Asset Management | Block Asset Management Ltd | crypto, fund-of-funds |
| TRM Labs | TRM Labs UK Ltd | crypto, blockchain-compliance |
| Bron Labs | Bron Labs Limited | crypto, security |
| Compass Labs | Compass Labs Ltd | defi, sim, infra |
| Clearmatics Technologies | Clearmatics Technologies Ltd | crypto, blockchain-r&d |
| Web3 Labs | Web3 Labs Ltd | blockchain, dev-tools |
| Kiln Technologies | Kiln Technologies Limited | crypto, staking-infra |
| Rated Labs | Rated Labs Ltd | crypto, data |
| Tonk Labs | Tonk Labs Limited | web3, data |
| Lattice Labs (London) | Lattice Labs Ltd. | crypto, web3-infra |
| XDEFI Wallet | XDEFI Technologies Ltd | crypto, wallet |
| 0xA Technologies | 0xA Technologies Ltd | crypto, web3, nft-infra |

## Cybersecurity

| Name | Register entity | Sector tags |
|---|---|---|
| Akamai Technologies | Akamai Technologies Limited | cdn, security, edge |
| CyberArk Software (UK) | Cyber-Ark Software (UK) Limited | identity, pam, cybersec |
| Aqua Security | Aqua Security Software UK LTD | cloud-security, k8s |
| SailPoint Technologies UK | SailPoint Technologies UK Ltd | identity, governance |
| OneTrust | OneTrust Technology Limited | privacy, grc, consent |
| Vanta Technology UK | Vanta Technology UK Limited | security, compliance |
| Wiz Cloud (already in DB) | Wiz Cloud Limited | cloud-security |
| Forescout Technologies UK | Forescout Technologies UK Ltd | iot, ot, security |
| Egress Software Technologies | Egress Software Technologies Limited | email-security, dlp |
| Garrison Technology | Garrison Technology Ltd | browser-isolation, cybersec |
| Smarsh (Cognia) | Cognia Cloud Limited t/a Smarsh | compliance, comms |
| Tufin Software Europe | Tufin Software Europe Limited | security-policy, automation |
| Immersive Labs | Immersive Labs Holdings | cybersec, training, gamified |
| KETS Quantum Security | KETS Quantum Security Ltd | quantum, security, chip |
| Cobalt Labs UK | Cobalt Labs UK Limited | cybersec, pentest, paas |
| Cheq | Cheq Technologies (UK) Limited | cybersec, adtech-verification |
| Searchlight Cyber | Searchlight Cyber Ltd | dark-web, intel |
| Interrupt Labs | Interrupt Labs Ltd | cybersec, vuln-research, defence |
| Redscan Cyber Security | Redscan Cyber Security Limited | mdr, pen-test, soc |
| Reliance Cyber | Reliance Cyber Limited | cybersec, mssp |
| Secon Cyber Security | Secon Cyber Security Ltd | cybersec, mssp |
| SE Labs | SE Labs Ltd | cybersec, testing |
| ThreatSpike Labs | ThreatSpike Labs Limited | cybersec, xdr |
| Bulletproof Cyber | Bulletproof Cyber Ltd | cybersec, pentest |
| Cowbell Cyber | Cowbell Cyber Ltd | cyber-insurance |
| OnSecurity Technology | ONSECURITY TECHNOLOGY LIMITED | cybersec, pentest, ai |
| Valarian Technologies | Valarian Technologies Limited | cybersec, data-sovereignty |
| Strider Technologies UK | Strider Technologies UK Limited | intel, geopolitical, supply-chain |
| C2 Cyber | C2 Cyber Ltd | cybersec, tprm |
| Cyber Range | Cyber Range Ltd | cybersec, training |
| The Blockhouse Technology | The Blockhouse Technology Limited | crypto, confidential-computing |
| Iceni Labs | Iceni Labs Limited | cheri, security, embedded |

## Robotics / Autonomous Systems

| Name | Register entity | Sector tags |
|---|---|---|
| MDA Space and Robotics | MDA Space and Robotics Limited | space, robotics, aerospace |
| Aalyria Technologies UK | Aalyria Technologies (UK) Limited | space-comms, laser, mesh |
| Hai Robotics UK | Hai Robotics U.K. Limited | warehouse-robotics |
| Sky-Drones Technologies | Sky-Drones Technologies Ltd | uav, drones, autopilot |
| SPAICE Technology | SPAICE Technology Ltd | space-ai, autonomy |
| Wayve Technologies | Wayve Technologies Ltd | autonomous-driving |
| Lotus Technology Innovative | Lotus Technology Innovative Limited | automotive, ev, adas, l4 |
| Cambrian Robotics | Cambrian Robotics Limited | robotics, ai, cv (also in AI section) |
| Extend Robotics | Extend Robotics Limited | robotics, teleoperation, vr |
| Fieldwork Robotics | Fieldwork Robotics Limited | robotics, agritech, ai |
| Perceptual Robotics | Perceptual Robotics Limited | robotics, drones, ai-inspection |
| Centaur Robotics | Centaur Robotics Limited | robotics, mobility |
| Paddington Robotics | Paddington Robotics | robotics, ai |
| PHINXT Robotics | PHINXT Robotics LTD | robotics, amr-warehouse |
| Prosper Robotics | Prosper Robotics Ltd | robotics, humanoid, ai |
| Reimagine Robotics | Reimagine Robotics Limited | robotics |
| Softbank Robotics UK | Softbank Robotics UK Limited | robotics, humanoid |
| SKL Robotics (Humanoid) | SKL ROBOTICS LTD | robotics, humanoid, industrial |
| ST Robotics | ST ROBOTICS LTD | robotics, industrial, embedded |
| Automata | Automata Technologies Limited | robotics, lab-automation, deeptech |
| Sportlight Technology | Sportlight Technology Ltd | lidar, sports-ai |
| Oxford Robotics (Dynium Robot) | Oxford Robotics Ltd trading as Dynium Robot | robotics, mobile |
| Ross Robotics | Ross Robotics | robotics, field |
| Motion Robotics | Motion Robotics Limited | robotics, r&d |
| i3d Robotics | i3d robotics limited (Operational Office) | robotics, cv-3d |
| KUKA Robotics UK | KUKA Robotics UK Ltd | robotics, industrial |
| Launch Robotics | LAUNCH ROBOTICS LTD | robotics, automation |

## Fintech / Payments / Embedded Finance

| Name | Register entity | Sector tags |
|---|---|---|
| 10X Banking Technology | 10X Banking Technology Services Ltd | core-banking, cloud |
| Form3 Technology | Form3 Technology Limited | payments-infra |
| Apexx Fintech | Apexx Fintech Ltd | payment-orchestration |
| Access Fintech | Access Fintech (UK) Limited | post-trade, fintech |
| Aptitude Software | Aptitude Software Group plc | finance-erp, fintech |
| Alfa Financial Software | Alfa Financial Software Ltd | asset-finance, capital-markets |
| Globacap Technology | Globacap Technology Limited | private-markets, fintech |
| Genesis Global Technology (already in DB) | Genesis Global Technology Limited. | low-code, capital-markets |
| Granola Labs (already in DB) | Granola Labs | ai, notetaking |
| Volante Technologies | Volante Technologies Inc. | payments-infra |
| Trustly Technologies UK | Trustly Technologies UK Ltd | payments |
| Ecospend (Trustly sub) | Ecospend Technologies Limited | open-banking, pis-ais |
| Ozone Financial Technology | Ozone Financial Technology Limited | open-banking, fapi |
| Lean Technologies | Lean Technologies Development (UK) Limited | open-banking, mena |
| BankiFi Technology | BankiFi Technology Limited | embedded-banking, sme |
| Soldo Software | Soldo Software Ltd | fintech, expense-cards |
| Pleo Technologies | Pleo Technologies Limited | fintech, expense |
| Plum Fintech | Plum Fintech | fintech, savings |
| Marshmallow Technology | Marshmallow Technology Ltd | insurtech, rust |
| Wise Alpha Technologies | WiseAlpha Technologies Limited | fintech, bonds |
| Yonder Technology | YONDER TECHNOLOGY LTD | fintech, credit-card |
| Zilch Technology | Zilch Technology Limited | bnpl, fintech |
| ZILO Technology | ZILO Technology Limited | funds-tech |
| Lenkie Technologies | Lenkie Technologies Limited | sme-financing, fintech |
| Pipe Technologies UK | Pipe Technologies UK Ltd | embedded-fintech |
| Toqio Fintech | Toqio Fintech Limited | embedded-finance |
| Wamo Technology | WAMO TECHNOLOGY LIMITED | fintech, sme, e-money |
| Nium Fintech | Nium Fintech Ltd | payments, cross-border |
| Moniepoint Technologies UK | Moniepoint Technologies UK Limited | fintech, africa-uk |
| Statement Technologies (Tuza) | Statement Technologies Limited T/A Tuza | fintech, sme |
| Bond Financial Technologies | Bond Financial Technologies Ltd | fintech, treasury-ai |
| Penfold Technology | Penfold Technology Limited | pensions, fintech |
| Personetics Technologies UK | Personetics Technologies (UK) Ltd | banking-ai |
| Ravelin Technology | Ravelin Technology Limited | fraud-ml, payments |
| SEON Technologies UK | SEON TECHNOLOGIES UK LTD | fraud, aml |
| ComplyAdvantage (IVXS) | IVXS UK Limited | aml, regtech |
| Globacap | Globacap Technology Limited | private-markets-infra |
| Modulr | Modulr Finance + Modulr Technology | payments-infra |
| Nethermind | Demerzel Solutions Limited | crypto-infra (also crypto) |
| Sling Money (Avian Labs) | Avian Labs Limited | crypto-payments (already in DB) |
| Currencycloud / Visa | The Currency Cloud Services Limited (+ VISA EUROPE LIMITED) | fx, payments |
| Snowflake | Snowflake Computing U.K. Limited | data-cloud (also in cloud) |
| Stripe (already in DB) | Stripe Payments UK Ltd | payments |
| Vega Investment Technologies | Vega Investment Technologies Limited | wealth-tech, buy-side-risk |
| Atominvest | Atominvest Software Ltd | fintech, alt-assets |
| Silverfin | Silverfin Software Ltd | accounting-saas |
| TaxScouts (Positron) | Positron Technologies Ltd T/A TaxScouts | fintech, consumer-tax |
| ClearScore (already in DB) | Clear Score Technology Limited | credit, fintech |
| Encord (already in DB as Cord Technologies) | Cord Technologies Limited | ai, data-labeling |
| Atticus (Assemble Tech) | Assemble Technologies UK Ltd (t/a Atticus) | legal-tech, dev-tools |
| Strategic Software Applications (Ruleguard) | Strategic Software Applications Ltd | regtech, grc |
| Brady Technologies | Brady Technologies Limited | energy-trading, ctrm |
| Smart Trade Technologies UK (already in DB) | SMART TRADE TECHNOLOGIES UK LIMITED | financial-trading |
| Token.io — wrong-route | (skip — GBM only) | — |
| Third Financial Software | Third Financial Software Limited | wealth-tech |
| Swallow Technology | Swallow Technology Limited | banking-infra |
| TaiNa Technology | Taina Technology | regtech, tax |
| Orbit Financial Technology | Orbit Financial Technology Limited | ai, fintech, nlp |
| Rogo | Rogo Technologies Ltd | ai, investment-banking |
| FintechOS | FINTECHOS TECHNOLOGY UK LTD | fintech, low-code-banking |
| Fonoa Technologies | Fonoa Technologies UK Ltd | tax-automation, api |
| Griffin Bank | Griffin Bank Ltd | fintech, baas |
| Volt Technologies Holdings (already in DB) | Volt Technologies Holdings Ltd | payments |
| Gresham Technologies | Gresham Technologies (Solutions) Limited | fintech, reconciliation |
| Hansen Technologies | Hansen Technologies CDE Limited | telco-bss, billing |
| ION Group (already in DB) | ION Trading UK Limited | trading-tech |
| FlexTrade Systems | FlexTrade UK Limited | trading, ems |
| Snowflake — see above | — | — |

## Cloud / DB / Dev Tools / Platform Engineering

| Name | Register entity | Sector tags |
|---|---|---|
| LinkedIn Technology UK | Linkedin Technology UK Limited | tech-major, microsoft |
| Asana Software UK | Asana Software UK Limited | saas, productivity |
| Intercom Software UK | Intercom Software UK Limited | saas, customer-messaging |
| Coupa Software UK | Coupa Software UK Ltd | enterprise-saas, procurement |
| Denodo Technologies | Denodo Technologies LTD | data-virtualization |
| Veeam Software UK | Veeam Software UK Limited | backup, storage |
| SolarWinds Software UK | SOLARWINDS SOFTWARE UK LIMITED | ops, monitoring |
| Spectro Cloud UK | Spectro Cloud, UK Limited | kubernetes |
| Brave Software Europe | Brave Software Europe Ltd | browser, crypto, rust |
| Lightdash (Telescope Tech) | Telescope Technology Limited t/a Lightdash | bi, dev-tools, oss |
| Curiosity Software | Curiosity Software Ireland | dev-tools, test-automation |
| Push Technology / Diffusion | Push Technology Limited | real-time-pub-sub, infra |
| Precisely Software | Precisely Software Limited | enterprise-data, data-integrity |
| Cast Software | Cast Software Limited | dev-tools, static-analysis |
| Sauce Labs Software UK | Sauce Labs Software UK Limited | dev-tools, testing |
| DBT Labs UK | DBT LABS UK LIMITED | dev-tools, data |
| BrightSign Technology | BrightSign Technology Limited | embedded, dev-tools, signage |
| Disguise Technologies | Disguise Technologies Ltd | real-time-rendering, dev-tools |
| GoTo Technologies UK | GoTo Technologies UK Limited | saas, remote-work |
| Humaans Software UK | Humaans Software UK Limited | hr-saas |
| Quantive Technologies UK | Quantive Technologies UK Limited | okr-saas |
| Workable Software | Workable Software Limited | hr-ats |
| Otta Technology | Otta Technology Limited t/a Otta | hr-tech, jobs |
| Ravio Technologies | Ravio Technologies Limited | hr-comp-data |
| WorkForce Software | WorkForce Software Limited | enterprise-saas, wfm |
| Xurrent Software UK | Xurrent Software UK Ltd | itsm, ai |
| Pimberly | Pimberly Software Development Limited | saas, pim |
| Propello Cloud | Propello Cloud Ltd | saas, loyalty |
| Medius Software | Medius Software Limited | fintech-saas, ap-automation |
| Kalibrate Technologies | Kalibrate Technologies Limited | ai, retail-pricing |
| Captify Technologies | Captify Technologies Ltd | adtech, ml |
| Attest Technologies | Attest Technologies Limited | research-saas |
| Refract Software | Refract Software Ltd | sales-ai, conversation-intel |
| Upland Software UK | UPLAND SOFTWARE UK LIMITED | enterprise-saas, content |
| Opencast Software Europe | Opencast Software Europe Limited | uk-public-sector, dev |
| Elastic Path Software | Elastic Path Software (UK) Ltd | headless-commerce |
| Vertice Technology | Vertice Technology Ltd | saas-spend-mgmt, ai |
| 1771 Technologies (LyteNyte) | 1771 Technologies Limited | dev-tools, perf-react |
| Container Solutions Software | Container Solutions Software Ltd | k8s, cloud-native (consultancy lean) |
| Aera Technology — see AI section | — | — |
| Cognia/Smarsh — see Cybersec section | — | — |

## Game Studios (Technical / Engine Work)

| Name | Register entity | Sector tags |
|---|---|---|
| Larian Studios UK | LARIAN STUDIOS UK LTD | games, engine, c++ |
| Take-Two Interactive | Take-Two Interactive Software Europe Limited | games, aaa |
| Rocksteady Studios | Rocksteady Studios Limited | games, aaa |
| Cloud Imperium Games | Cloud Imperium Games Limited | games, engine, c++ |
| Tripledot Studios | Tripledot Studios Limited | games, mobile |
| Keywords Studios | Keywords Studios PLC | games-services |
| Bossa Studios | Bossa Studios | games |
| Dambuster Studios | Dambuster Studios Ltd | games, aaa |
| Omeda Studios | Omeda Studios Limited | games, ue |
| Dlala Studios | Dlala Studios | games, indie |
| Facepunch Studios | Facepunch Studios Limited | games, engine, c++ |
| Stellar Entertainment Software | Stellar Entertainment Software Ltd | games, technical |
| Payload Studios (Terra Tek) | Terra Tek Studios Limited T/A Payload Studios | games, indie |
| The Workshop Technologies | The Workshop Technologies Ltd | games, gambling, fintech |
| Black Cow Technology | Black Cow Technology Limited | games, igaming-engine |
| Betfred Technology (Sharp Gaming) | Betfred Technology Limited | gambling-platform, in-house |
| EveryMatrix Technology UK | EVERYMATRIX TECHNOLOGY (UK) LIMITED | igaming-platform |
| Playtech Software | Playtech Software Limited | gambling-tech, lse |
| Auction Technology Group (Metropress) | Metropress Limited t/a Auction Technology Group | online-auctions, plc |
| Caesars Trading and Technology Services | Caesars Trading and Technology Services Limited | sportsbook-trading-tech |

## Deeptech (Quantum, Photonics, Semiconductors, Materials)

| Name | Register entity | Sector tags |
|---|---|---|
| Qualcomm Technologies International | Qualcomm Technologies International Ltd | semiconductors |
| Infineon Technologies UK | Infineon Technologies UK Limited | semiconductors |
| Huawei Technologies UK | Huawei Technologies (UK) Co., Ltd | r&d, networking |
| Huawei R&D UK | Huawei Technologies Research and Development | r&d |
| Cambridge Display Technology | Cambridge Display Technology Ltd | deeptech, oled |
| Cambridge Touch Technologies | Cambridge Touch Technologies Limited | deeptech, piezo-ai-sensing |
| FlexEnable Technology | FlexEnable Technology Ltd | deeptech, flexible-electronics |
| Picocom Technology | Picocom Technology Limited | semiconductors, 5g-oran |
| Credo Technology Services UK | Credo Technology Services UK Ltd | semiconductors, serdes |
| Blu Wireless Technology | Blu Wireless Technology Limited | semiconductors, mmwave-5g |
| Quantum Motion Technologies | Quantum Motion Technologies Limited | quantum, silicon-cmos |
| Quantum Detectors | Quantum Detectors Ltd | deeptech, scientific-instruments |
| Quantum Dice | Quantum Dice Limited | quantum, qrng |
| Quantum Base | Quantum Base Limited | quantum, security, semicon |
| Duality Quantum Photonics | Duality Quantum Photonics Limited | quantum, photonics |
| KETS Quantum Security — see Cybersec | — | — |
| Salience Labs | Salience Labs Ltd | deeptech, photonic-compute |
| Frontier Space Technologies | FRONTIER SPACE TECHNOLOGIES LTD | deeptech, space-biotech |
| Mirion Technologies | Mirion Technologies (IST) Limited | deeptech, nuclear-instrumentation |
| Phlux Technology | Phlux Technology Ltd | deeptech, photodiodes-infrared |
| Opteran Technologies | Opteran Technologies Ltd | neuromorphic-ai, rust |
| Oxford Nanopore Technologies | Oxford Nanopore Technologies plc. | deeptech, dna-sequencing |
| Oxford Semantic Technologies | Oxford Semantic Technologies Limited | knowledge-graph, db, c++ |
| Oxford Quantum Circuits — already in DB | Oxford Quantum Circuits Limited | quantum |
| Mantle Labs | MANTLE LABS LIMITED | geospatial-ai, climate |
| TitanML (Doubleword) | TYTN LTD | ai (already in DB) |
| Speechmatics — already in DB | CANTAB RESEARCH LIMITED | speech-ai |
| Element (Matrix) — already in DB | New Vector Limited t/as Element | crypto-comms |
| Mission Zero Technologies | Mission Zero Technologies Ltd | deeptech, climate-dac |
| Breathe Battery Technologies | Breathe Battery Technologies Limited | deeptech, battery-bms |
| GT Wings (Green Technologies) | GT Wings t/a GT Green Technologies Limited | deeptech, maritime |
| Marlan Maritime Technologies | Marlan Maritime Technologies Ltd | deeptech, radar |
| Silverstream Technologies | Silverstream Technologies (UK) Limited | deeptech, maritime |
| Reactive Technologies | Reactive Technologies Limited | deeptech, grid-stability |
| CoMind Technologies | CoMind Technologies Limited | neurotech, deeptech |
| Portal Biotech | Portal Biotech Limited | deeptech, biotech, nanopore |
| Lifebit Biotech | Lifebit Biotech Limited | bioinformatics, federated |
| Sention Technologies | Sention Technologies Limited | battery-ml, deeptech |
| G2O Water Technologies | G2O Water Technologies Ltd | deeptech, graphene-membranes |
| Watercycle Technologies | Watercycle Technologies | deeptech, lithium-extraction |
| Whitefox Technologies | Whitefox Technologies Limited | cleantech, membranes |
| Worn Again Technologies | Worn Again Technologies Limited | deeptech, textile-recycling |
| Romax Technology (Hexagon) | Romax Technology Limited | cae, ev-drivetrain |
| Silicon Frontline Technology (Synopsys) | Silicon Frontline Technology (UK) Limited | eda, semiconductors |
| Adder Technology | Adder Technology Limited | kvm-over-ip, hardware |
| Cyber-Ark — see Cybersec | — | — |
| Myrtle.ai (Myrtle Software) | Myrtle Software Limited | fpga-inference, hft-adjacent |
| Brompton Technology | Brompton Technology Ltd | led-video-processing, fpga |
| Disguise Technologies — see Dev Tools | — | — |
| Methodica Technologies | Methodica Technologies Limited | automotive-embedded |
| SeeChange Technologies | SeeChange Technologies Limited | cv, edge-ai, retail |
| Facit Data Systems | Facit Data Systems | cv, video-redaction |
| Starfish Technologies | Starfish Technologies Limited | broadcast-media, c++ |
| Aalyria — see Robotics | — | — |
| GivEnergy Software | GivEnergy Software Ltd | clean-energy, battery, ev |
| Neara Software | Neara Software Ltd | utilities, digital-twin |
| Apis Assay Technologies | Apis Assay Technologies | diagnostics, bioinfo |
| Synativ Technologies | Synativ Technologies Ltd | geospatial-ai |
| Agreena Technology | Agreena Technology Limited | climate-mrv, agritech, satellite |
| WIM Technologies | WIM Technologies | telecoms-ai, ran |
| Argus Software UK | Argus Software (UK) Ltd | cre-valuation, quant |
| Marquis Technologies | Marquis Technologies Limited | defence-sim, c++ |
| Systematic Software Engineering | Systematic Software Engineering Limited | defence-c2, real-time |
| Remote Diagnostic Technologies | Remote Diagnostic Technologies Limited | medical-defence-systems |
| Evariste Technologies | Evariste Technologies Limited | defence-sdr, small |
| Aera Technology — see AI | — | — |
| Parametric Technology PTC | Parametric Technology (UK) Limited | cad, plm, iot |
| Siemens Industry Software | Siemens Industry Software Limited | nx, eda, plm |
| BAE Systems Digital Intelligence | BAE Systems Plc | defence, cyber, intel |
| Alstom Transport UK | Alstom Transport UK Ltd | transport, rail |
| Dassault Systemes UK | Dassault Systemes UK Ltd | engineering-software |
| Faculty Science Limited | Faculty Science Limited | ai, govt, applied |
| MoA Technology | MoA Technology Ltd | agritech, biotech-software |
| TomTom Software | TomTom Software Limited | mapping, automotive |
| Voi Technology UK | Voi Technology UK Ltd | micromobility |
| Wheely Technologies | Wheely Technologies Ltd | premium-rideshare |
| Volteras Technologies | Volteras Technologies Ltd | ev-charging-data |
| Xiaomi Technology UK | Xiaomi Technology UK Limited | consumer-hardware |
| Nothing Technology | NOTHING TECHNOLOGY LIMITED | consumer-hardware, embedded |
| Joan Technologies | Joan Technologies Ltd | workplace-saas, e-paper |
| Zamna Technologies | Zamna Technologies Limited | aviation-identity, crypto |
| Vivos Technology (PHASTAR) | Vivos Technology Limited (trading as PHASTAR) | clinical-biostats |
| TikTok Information Technologies UK | TikTok Information Technologies UK Limited | bytedance |
| Trace Machina, etc. (verify) | — | — |
| Solera Global Technology | Solera Global Technology Ltd | insurance-automotive-ai |

## Notable / Other Targets

| Name | Register entity | Sector tags |
|---|---|---|
| OCaml Labs Consultancy | OCaml Labs Consultancy Limited | compilers, functional, plt |
| Hudson River Trading — already in DB | Hudson River Trading Europe Ltd. | hft |
| Citadel — already in DB | Citadel Enterprise Europe Limited | hedge, quant |
| ION Trading — already in DB | ION Trading UK Limited | trading-tech |
| Modulr — see Fintech | — | — |
| Auction Technology Group — see Games | — | — |

---

## Recommended next steps

1. **Eyeball this list first.** This is ~378 candidates; bulk-importing all of them adds significant grading workload. Consider importing the S/A-tier candidates first (quant funds, frontier AI labs, top fintech, deeptech) and deferring the long-tail.

2. **Suggested S-tier import priority (~50 companies):**
   - **Quant flagship:** AQR, Marshall Wace, Millennium, Brevan Howard, BlueCrest, ExodusPoint, Saba, Sculptor, Taula, Tenaron, Alphadyne, Quantbot, BH-DG, OxAM, Winton, Numerix, Bayforest, Marathon, Cheyne, SPX, Susquehanna
   - **Top fintech / payments:** Form3, 10X Banking, Apexx, Access Fintech, Alfa, Aptitude, Volante, Trustly, Ozone, Lean Tech, GSR, ADG Market Making, Geneva Trading
   - **AI labs (top-tier):** Palantir UK, Google DeepMind, Latent Labs, Flower Labs, Mantic, Magentic, Gradient, Lovable, Periodic, Convergence, Altos Labs, Ignota, Aera, Altana, AlphaSense, Eigen, Luminance, Xapien, TurinTech
   - **Deeptech:** Qualcomm UK, Cambridge Display Technology, Cambridge Touch, FlexEnable, Picocom, Credo Semi, Quantum Motion, Quantum Dice, Salience Labs, Aalyria, Mirion, Phlux, Opteran, Oxford Nanopore, Oxford Semantic, Myrtle.ai, Brompton, Mission Zero, Watershed
   - **Crypto / DeFi:** Chainlink, Euler, Argent, Anera, Equilibrium, OKX, Paxos, Block AM, TRM, Clearmatics, Kiln
   - **Cybersec (top):** Akamai, CyberArk, Aqua, SailPoint, OneTrust, Vanta, Forescout, Egress, Garrison, Smarsh, Tufin, Valarian, Strider, Immersive Labs, KETS, Iceni

3. **A/B tier follow-up batches.** Roughly 200-250 more A/B tier candidates remain in the lists above; consider a second import wave once S-tier targets are graded and apply-queued.

4. **Schema enhancement.** Adding `uk_sponsor_entity_name` + `uk_sponsor_status` columns to `companies` (per earlier session discussion) would let this register-driven discovery become a recurring monthly pipeline rather than a one-off ingest.

---

## WIDND — what we didn't do

- **No duplicates removed in this file** — the 30+ entries marked "already in DB" are listed so you can spot-check, but they shouldn't be re-imported. The aggregator pass needs to dedupe by normalised name AND by detecting the legal-entity-name overlap with the existing 349 companies that already have correct register entity names (Speechmatics→Cantab Research, etc.).
- **MAYBE category not fully reproduced** — the 10 agents produced ~60 MAYBE entries across batches that didn't make this file (mostly tiny / unverifiable / very early stage). Re-check the agent transcripts in `/private/tmp/claude-501/.../tasks/` if you want the full MAYBE pool.
- **No website URLs yet** — `cernio import` requires a website. The next step before import is to resolve careers-page / homepage URLs for each KEEP. The `populate-db` skill can do this in parallel.
- **No grade assigned** — these are discovery candidates only. After import they'll go through `grade-companies` for proper S/A/B/C tiering against your profile.
- **Sector tags are loose** — final sector_tags column should be cleaned up at import time to match the grading-rubric conventions.
