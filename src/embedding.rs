use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;
use hf_hub::{api::sync::Api, Repo, RepoType};

pub struct EmbeddingEngine {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu; // Stick to CPU for simplicity in CLI
        
        let api = Api::new()?;
        let model_id = "sentence-transformers/all-MiniLM-L6-v2".to_string();
        let repo = api.repo(Repo::with_revision(model_id, RepoType::Model, "main".to_string()));
        
        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config: Config = serde_json::from_reader(std::fs::File::open(config_filename)?)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow!(e))?;
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self { model, tokenizer, device })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let tokens = self.tokenizer.encode(text, true).map_err(|e| anyhow!(e))?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        let attention_mask = Tensor::new(tokens.get_attention_mask(), &self.device)?.unsqueeze(0)?.to_dtype(DType::F32)?;
        
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;
        
        // Mean pooling with attention mask
        let mask_expanded = attention_mask.unsqueeze(2)?.broadcast_as(embeddings.shape())?;
        let sum_embeddings = (embeddings * &mask_expanded)?.sum(1)?;
        let sum_mask = mask_expanded.sum(1)?;
        let mean_pooled = sum_embeddings.broadcast_div(&sum_mask)?;

        // L2 normalization
        let norm = mean_pooled.sqr()?.sum_keepdim(1)?.sqrt()?;
        let final_embeddings = mean_pooled.broadcast_div(&norm)?;
        
        let vec = final_embeddings.squeeze(0)?.to_vec1()?;
        Ok(vec)
    }
}
