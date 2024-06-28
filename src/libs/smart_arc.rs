use std::{
	fmt::{self, Debug},
	hash::{Hash, Hasher},
	ops::Deref,
	sync::Arc,
};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub struct SmartArc<T: ?Sized>(pub Arc<T>);

impl<T: Sized> SmartArc<T> {
	pub fn new(data: T) -> Self {
		Self(Arc::new(data))
	}
}

impl<T: ?Sized> Clone for SmartArc<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T: ?Sized> PartialEq for SmartArc<T> {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<T: ?Sized> Eq for SmartArc<T> {}

impl<T: ?Sized> Hash for SmartArc<T> {
	fn hash<H>(&self, hasher: &mut H)
	where
		H: Hasher,
	{
		// Voodoo magic, but basically we're hashing using the numeric value of the
		// pointer of the Arc
		hasher.write_usize(Arc::as_ptr(&self.0) as *const () as usize);
	}
}

impl<T: ?Sized> Debug for SmartArc<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple(std::any::type_name::<Self>()).finish()
	}
}

impl<T: ?Sized> Deref for SmartArc<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl<T: ?Sized> AsRef<T> for SmartArc<T> {
	fn as_ref(&self) -> &T {
		self.0.as_ref()
	}
}
