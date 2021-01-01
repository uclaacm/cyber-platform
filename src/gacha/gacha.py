from flask import Flask, request, render_template
from flask_sqlalchemy import SQLAlchemy
import os

app = Flask(__name__)
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ['URI']
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
db = SQLAlchemy(app)

from schema import *

@app.route('/rewards')
def gatcha():
    return render_template('rewards.html')

@app.route('/rewards/redeem', methods=['POST'])
def redeem():
    reward_type = request.form.get("type")
    if reward_type is None:
        return "Error"

    return "hey"

if __name__ == '__main__':
    app.run()