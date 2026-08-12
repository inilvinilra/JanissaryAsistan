const fs = require('fs');
const path = require('path');

const DIRECTORIES_TO_IGNORE = ['node_modules', 'target', '.git', 'dist', '.astro'];
const EXTENSIONS_TO_INCLUDE = ['.ts', '.tsx', '.rs', '.astro', '.sql', '.toml', '.md'];

function replaceInFile(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    if (content.includes('KYS') || content.includes('kys') || content.includes('Kys')) {
      // Sadece görünür metinleri ve spesifik logları değiştirelim, 
      // klasör yollarını veya dosya isimlerini (kys-app, kys-engine) bozmamak için dikkat edelim.
      // Biz doğrudan 'KYS ' veya 'KYS-' veya benzeri kullanımlara odaklanacağız ama tam metin değiştirme isteniyor.
      // "KYS" (büyük harf) geçişlerini "JanissaryAsistan" yapalım.
      let newContent = content.replace(/KYS/g, 'JanissaryAsistan');
      
      if (content !== newContent) {
        fs.writeFileSync(filePath, newContent, 'utf8');
        console.log(`Updated: ${filePath}`);
      }
    }
  } catch (e) {
    console.error(`Error reading ${filePath}`, e);
  }
}

function walkDir(dir) {
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat.isDirectory()) {
      if (!DIRECTORIES_TO_IGNORE.includes(file)) {
        walkDir(fullPath);
      }
    } else {
      const ext = path.extname(fullPath);
      if (EXTENSIONS_TO_INCLUDE.includes(ext)) {
        replaceInFile(fullPath);
      }
    }
  }
}

walkDir(path.join(__dirname, 'kys-app'));
walkDir(path.join(__dirname, 'kys-engine'));
walkDir(__dirname); // to catch md files in root

console.log('Renaming complete.');
