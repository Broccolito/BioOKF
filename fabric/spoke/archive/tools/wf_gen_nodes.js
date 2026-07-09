export const meta = {
  name: 'gen-biookf-test-nodes',
  description: 'Generate 100 real example entities for every BioOKF node type (for stress testing)',
  phases: [{ title: 'Generate', detail: 'one agent per BioOKF type' }],
};

// The 28 BioOKF types with a hint of scope + subtype examples (from SCHEMA.md).
const TYPES = [
  ['Gene', 'heritable loci / genes (HGNC symbols, incl. ncRNA/miRNA genes)', 'protein_coding/ncRNA/miRNA'],
  ['Molecule', 'proteins, drugs, small molecules, metabolites, antibodies, ions, RNA species', 'protein/drug/small_molecule/metabolite/antibody/ion/rna'],
  ['MolecularClass', 'drug classes, protein families/domains, gene sets, chemical classes', 'pharmacologic/protein_family/protein_domain/gene_set/chemical_class'],
  ['Variant', 'sequence variants: SNVs (rsIDs), indels, CNVs, alleles, haplotypes, protein-change notation', 'snv/indel/cnv/allele/haplotype/fusion'],
  ['SequenceFeature', 'reference functional/regulatory elements: enhancers, promoters, TFBS, CpG islands', 'enhancer/promoter/silencer/tfbs/cpg_island'],
  ['Structure', '3D atomic structures (PDB entries, resolved complexes)', 'xray/cryo_em/nmr/predicted'],
  ['Anatomy', 'organs, tissues, body regions, organelles, body fluids', 'organ/tissue/subcellular/body_fluid'],
  ['CellType', 'cell types/states/lines/organoids', 'cell_type/cell_state/cell_line/organoid'],
  ['Organism', 'species, strains, pathogens, microbes (scientific binomials)', 'species/strain/pathogen'],
  ['BiologicalPathway', 'pathways, reactions, signaling cascades, GO biological processes', 'pathway/reaction/signaling/go_bp'],
  ['BiologicalFunction', 'molecular functions (GO-MF): catalytic/binding/transporter activities', 'catalytic/binding/transporter'],
  ['Disease', 'diseases, syndromes, infections, cancers (diagnoses)', 'infection/neoplasm/syndrome/injury'],
  ['Phenotype', 'symptoms, signs, side effects, qualitative traits', 'symptom/sign/side_effect/trait'],
  ['BiomedicalMeasure', 'measurable variables: lab tests, vitals, scores, biomarkers, imaging findings', 'lab_test/vital/score/biomarker/imaging_finding'],
  ['MethodOrProcedure', 'clinical procedures, assays, pipelines, software, statistical methods', 'surgical/imaging/lab_assay/software/statistical_method'],
  ['Exposure', 'behavioral/environmental/occupational/dietary exposures', 'behavioral/environmental/occupational/dietary'],
  ['SocialFactor', 'social determinants of health (income, education, housing, access to care)', 'economic/education/housing/healthcare_access'],
  ['Food', 'food items, food groups, dietary products', 'food_item/food_group/dietary_product'],
  ['Device', 'devices, implants, prostheses, instruments, reagents', 'implant/prosthesis/instrument/reagent'],
  ['MaterialSample', 'biospecimens: serum, biopsy, aliquot, cell-line stock', 'serum/biopsy/aliquot/cell_line_stock'],
  ['Publication', 'papers, preprints, slide decks, notes (realistic titles)', 'article/preprint/tweet/lab_notebook'],
  ['Study', 'clinical trials, cohort studies, GWAS, registries (realistic names)', 'rct/cohort/gwas/registry'],
  ['Dataset', 'datasets, omics matrices, image sets, knowledge bases (realistic names)', 'table/omics_matrix/image_collection/knowledge_base'],
  ['Agent', 'people/authors, labs, orgs, companies, regulators', 'person/lab/organization/company/regulator'],
  ['Population', 'groups of people: cohorts, ancestries, demographic groups', 'cohort/ancestry/demographic'],
  ['GeographicLocation', 'countries, regions, places', 'country/region/place'],
  ['Concept', 'abstract scores/classifications/units/ontology terms', 'unit/classification/ontology_term'],
  ['Other', 'genuinely-none-of-the-above biomedical concepts', 'none'],
];

const SCHEMA = {
  type: 'object', additionalProperties: false, required: ['type', 'nodes'],
  properties: {
    type: { type: 'string' },
    nodes: {
      type: 'array', minItems: 90,
      items: {
        type: 'object', additionalProperties: false, required: ['name', 'subtype'],
        properties: {
          name: { type: 'string', description: 'the human-readable entity name as written in biomedical text' },
          subtype: { type: 'string' },
        },
      },
    },
  },
};

function prompt(type, scope, subs) {
  return `Generate exactly 100 REAL, distinct example entities of the BioOKF type "${type}" (${scope}).
Write each name exactly as it appears in biomedical literature — real gene symbols, real drug names,
real disease names, real anatomy terms, real species binomials, etc. NOT placeholders, NOT invented.

Requirements:
- 100 entries, all real and DISTINCT (no duplicates, no near-duplicates).
- DIVERSE across the type's range and across the subtypes: ${subs}.
- Vary naming: include common names, some acronyms, some full names, a few older/alternate names —
  this is a robustness test, so realistic variety matters.
- Each entry: {name, subtype}. subtype is a lowercase token from the examples above (or a sensible
  one you coin). For provenance types (Publication/Study/Dataset/Agent) write realistic-looking
  titles/names.

Return ONLY the structured object {type, nodes:[100x {name, subtype}]}.`;
}

phase('Generate');
const results = await parallel(TYPES.map(([t, scope, subs]) => () =>
  agent(prompt(t, scope, subs), { label: `gen:${t}`, phase: 'Generate', schema: SCHEMA })
    .then(r => r ? { type: t, nodes: (r.nodes || []).slice(0, 100) } : null)));
const out = {};
for (const r of (results || []).filter(Boolean)) out[r.type] = r.nodes;
const total = Object.values(out).reduce((a, n) => a + n.length, 0);
log(`generated ${total} nodes across ${Object.keys(out).length} types`);
return out;
