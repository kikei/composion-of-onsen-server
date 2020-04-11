db.auth('admin', 'admin');

db = db.getSiblingDB('onsen');
db.createUser({
  user: 'webapp',
  pwd: 'webapp',
  roles: [
    {
      role: 'dbOwner',
      db: 'onsen'
    }
  ]
});
