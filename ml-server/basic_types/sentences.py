import dataclasses

@dataclasses.dataclass
class Sentence:
    start_pos: int
    end_pos: int
    sentence: str
