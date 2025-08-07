// Vercel serverless function for getting AI backends
export default async function handler(req, res) {
  if (req.method === 'GET') {
    // Simple static backend list for demo
    const backends = [
      {
        id: "openai",
        name: "OpenAI GPT",
        backend_type: "api",
        enabled: true,
        config: {}
      },
      {
        id: "gemini", 
        name: "Google Gemini",
        backend_type: "api", 
        enabled: true,
        config: {}
      }
    ];
    
    res.status(200).json(backends);
  } else {
    res.status(405).json({ error: 'Method not allowed' });
  }
}