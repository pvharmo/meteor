from fastapi import FastAPI
from synsets import f_hypothesis_syns
from nltk.stem.porter import PorterStemmer

app = FastAPI()

@app.get("/")
def read_root():
    return {"Hello": "World"}

@app.get("/synsets/{word}")
def synsets(word: str):
    return f_hypothesis_syns(word)

@app.get("/stemmer/{word}")
def stemmer(word: str):
    porter_stemmer = PorterStemmer()
    return porter_stemmer.stem(word)
