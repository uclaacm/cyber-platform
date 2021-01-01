from datetime import datetime
from gacha import db

class Team(db.Model):
  __tablename__ = 'team'
  __table_args__ = {'schema': 'scrap'}

  id = db.Column(db.Integer, primary_key=True)
  sessions = db.relationship('Session', backref='team_lookup')
  name = db.Column(db.String(), unique=True, nullable=False)
  discord = db.Column(db.String(), unique=True, nullable=False)
  hash = db.Column(db.String(), nullable=False)
  solves = db.Column(db.Integer, default=0)
  score = db.Column(db.Integer, default=0)
  redeemed_score = db.Column(db.Integer, default=0)
  premium_tickets = db.Column(db.Integer, default=0)
  isadmin = db.Column(db.Boolean, default=False)
  submit = db.Column(db.DateTime, default=datetime.utcnow)


  def __init__(self, name, discord, hash):
    self.name = name
    self.discord = discord
    self.hash = hash

  def __repr__(self):
    return f"<Team {self.name}>"

class Prize(db.Model):
  __tablename__ = 'prize'
  __table_args__ = {'schema': 'scrap'}

  id = db.Column(db.String(), nullable=False, primary_key=True)
  team = db.Column(db.Integer, db.ForeignKey('scrap.team.id', ondelete="CASCADE"), nullable=False)
  prize = db.Column(db.String(), nullable=False)

  def __init__(self, team, prize):
    self.id = '{}{}'.format(team, prize)
    self.team = team
    self.prize = prize

  def __repr__(self):
    return f"<Prize {self.team}>"

class Raffle(db.Model):
  __tablename__ = 'raffle'
  __table_args__ = {'schema': 'scrap'}

  team = db.Column(db.Integer, db.ForeignKey('scrap.team.id', ondelete="CASCADE"), nullable=False)
  # id doesn't exist in the actual table, this is a hack so sqlalchemy doesn't get mad
  id = db.Column(db.Integer, primary_key=True) 

  def __init__(self, team):
    self.team = team

  def __repr__(self):
    return f"<Raffle {self.team}>"

class Session(db.Model):
  __tablename__ = 'session'
  __table_args__ = {'schema': 'scrap'}

  cookie = db.Column(db.Text, primary_key=True)
  team = db.Column(db.Integer, db.ForeignKey('scrap.team.id', ondelete="CASCADE"), nullable=False)

  def __init__(self, team, cookie):
    self.team = team
    self.cookie = cookie

  def __repr__(self):
    return f"<Session {self.cookie}>"