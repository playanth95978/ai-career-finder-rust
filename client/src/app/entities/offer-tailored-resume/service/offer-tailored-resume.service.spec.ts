import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IOfferTailoredResume } from '../offer-tailored-resume.model';
import {
  sampleWithFullData,
  sampleWithNewData,
  sampleWithPartialData,
  sampleWithRequiredData,
} from '../offer-tailored-resume.test-samples';

import { OfferTailoredResumeService, RestOfferTailoredResume } from './offer-tailored-resume.service';

const requireRestSample: RestOfferTailoredResume = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
};

describe('OfferTailoredResume Service', () => {
  let service: OfferTailoredResumeService;
  let httpMock: HttpTestingController;
  let expectedResult: IOfferTailoredResume | IOfferTailoredResume[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(OfferTailoredResumeService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  describe('Service methods', () => {
    it('should find an element', () => {
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.find(123).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should create a OfferTailoredResume', () => {
      const offerTailoredResume = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(offerTailoredResume).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a OfferTailoredResume', () => {
      const offerTailoredResume = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(offerTailoredResume).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a OfferTailoredResume', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of OfferTailoredResume', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a OfferTailoredResume', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addOfferTailoredResumeToCollectionIfMissing', () => {
      it('should add a OfferTailoredResume to an empty array', () => {
        const offerTailoredResume: IOfferTailoredResume = sampleWithRequiredData;
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing([], offerTailoredResume);
        expect(expectedResult).toEqual([offerTailoredResume]);
      });

      it('should not add a OfferTailoredResume to an array that contains it', () => {
        const offerTailoredResume: IOfferTailoredResume = sampleWithRequiredData;
        const offerTailoredResumeCollection: IOfferTailoredResume[] = [
          {
            ...offerTailoredResume,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing(offerTailoredResumeCollection, offerTailoredResume);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a OfferTailoredResume to an array that doesn't contain it", () => {
        const offerTailoredResume: IOfferTailoredResume = sampleWithRequiredData;
        const offerTailoredResumeCollection: IOfferTailoredResume[] = [sampleWithPartialData];
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing(offerTailoredResumeCollection, offerTailoredResume);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(offerTailoredResume);
      });

      it('should add only unique OfferTailoredResume to an array', () => {
        const offerTailoredResumeArray: IOfferTailoredResume[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const offerTailoredResumeCollection: IOfferTailoredResume[] = [sampleWithRequiredData];
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing(offerTailoredResumeCollection, ...offerTailoredResumeArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const offerTailoredResume: IOfferTailoredResume = sampleWithRequiredData;
        const offerTailoredResume2: IOfferTailoredResume = sampleWithPartialData;
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing([], offerTailoredResume, offerTailoredResume2);
        expect(expectedResult).toEqual([offerTailoredResume, offerTailoredResume2]);
      });

      it('should accept null and undefined values', () => {
        const offerTailoredResume: IOfferTailoredResume = sampleWithRequiredData;
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing([], null, offerTailoredResume, undefined);
        expect(expectedResult).toEqual([offerTailoredResume]);
      });

      it('should return initial array if no OfferTailoredResume is added', () => {
        const offerTailoredResumeCollection: IOfferTailoredResume[] = [sampleWithRequiredData];
        expectedResult = service.addOfferTailoredResumeToCollectionIfMissing(offerTailoredResumeCollection, undefined, null);
        expect(expectedResult).toEqual(offerTailoredResumeCollection);
      });
    });

    describe('compareOfferTailoredResume', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareOfferTailoredResume(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 5742 };
        const entity2 = null;

        const compareResult1 = service.compareOfferTailoredResume(entity1, entity2);
        const compareResult2 = service.compareOfferTailoredResume(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 5742 };
        const entity2 = { id: 2726 };

        const compareResult1 = service.compareOfferTailoredResume(entity1, entity2);
        const compareResult2 = service.compareOfferTailoredResume(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 5742 };
        const entity2 = { id: 5742 };

        const compareResult1 = service.compareOfferTailoredResume(entity1, entity2);
        const compareResult2 = service.compareOfferTailoredResume(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
