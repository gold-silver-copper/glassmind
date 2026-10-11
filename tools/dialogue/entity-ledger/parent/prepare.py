from pathlib import Path
import hashlib
import json

HERE = Path(__file__).resolve().parent
cases = []
names = ['Amber Kite Observatory', 'Cobalt Finch Gallery', 'Isla Verde Pavilion',
         'Étoile Lantern Museum', 'Orchid Glass Atrium', 'Saffron Bell Theatre']
def turn(speaker, utterance):
    return dict(speaker=speaker, utterance=utterance)
for i, name in enumerate(names):
    opening = [turn('USER', 'Can you recommend an attraction to visit?'),
               turn('SYSTEM', f'I recommend {name}. It is a museum worth visiting.')]
    for delayed in [False, True]:
        turns = list(opening)
        if delayed:
            turns += [turn('USER', 'What will the weather be like there?'),
                      turn('SYSTEM', 'The temperature will be 22 degrees and there is no chance of rain.')]
        turns += [turn('USER', 'Please book a cab there for two people.')]
        cases.append(dict(name=f'offer_{i}_' + ('delayed' if delayed else 'immediate'),
                          kind='positive', expected=name, delayed=delayed, turns=turns))
for i, request in enumerate([
    'Please book a cab to one of those two places.',
    'I have not chosen between those places, so do not book a cab.',
    'Do not book a cab to that place.',
    'I do not want to visit that place. Do not book a cab.'
]):
    source = ('The two options are Amber Kite Observatory and Cobalt Finch Gallery.'
              if i < 2 else 'I recommend Amber Kite Observatory.')
    turns = [turn('USER', 'Can you recommend an attraction to visit?'),
             turn('SYSTEM', source), turn('USER', 'What will the weather be like?'),
             turn('SYSTEM', 'It will be sunny and warm.'), turn('USER', request)]
    cases.append(dict(name=f'control_{i}', kind='control', turns=turns))
(HERE/'cases.json').write_text(json.dumps(cases, ensure_ascii=False, indent=2)+'\n')
(HERE/'inputs.json').write_text(json.dumps([c['turns'] for c in cases], ensure_ascii=False)+'\n')
(HERE/'freeze.json').write_text(json.dumps({n:hashlib.sha256((HERE/n).read_bytes()).hexdigest()
    for n in ['cases.json','inputs.json','prepare.py','PLAN.md']},indent=2)+'\n')
