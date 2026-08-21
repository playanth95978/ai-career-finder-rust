import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { ICandidateProfile } from '../candidate-profile.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../candidate-profile.test-samples';

import { CandidateProfileService, RestCandidateProfile } from './candidate-profile.service';

const requireRestSample: RestCandidateProfile = {
  ...sampleWithRequiredData,
  embeddedAt: sampleWithRequiredData.embeddedAt?.toJSON(),
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
  updatedAt: sampleWithRequiredData.updatedAt?.toJSON(),
};

describe('CandidateProfile Service', () => {
  let service: CandidateProfileService;
  let httpMock: HttpTestingController;
  let expectedResult: ICandidateProfile | ICandidateProfile[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(CandidateProfileService);
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

    it('should create a CandidateProfile', () => {
      const candidateProfile = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(candidateProfile).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a CandidateProfile', () => {
      const candidateProfile = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(candidateProfile).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a CandidateProfile', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of CandidateProfile', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a CandidateProfile', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addCandidateProfileToCollectionIfMissing', () => {
      it('should add a CandidateProfile to an empty array', () => {
        const candidateProfile: ICandidateProfile = sampleWithRequiredData;
        expectedResult = service.addCandidateProfileToCollectionIfMissing([], candidateProfile);
        expect(expectedResult).toEqual([candidateProfile]);
      });

      it('should not add a CandidateProfile to an array that contains it', () => {
        const candidateProfile: ICandidateProfile = sampleWithRequiredData;
        const candidateProfileCollection: ICandidateProfile[] = [
          {
            ...candidateProfile,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addCandidateProfileToCollectionIfMissing(candidateProfileCollection, candidateProfile);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a CandidateProfile to an array that doesn't contain it", () => {
        const candidateProfile: ICandidateProfile = sampleWithRequiredData;
        const candidateProfileCollection: ICandidateProfile[] = [sampleWithPartialData];
        expectedResult = service.addCandidateProfileToCollectionIfMissing(candidateProfileCollection, candidateProfile);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(candidateProfile);
      });

      it('should add only unique CandidateProfile to an array', () => {
        const candidateProfileArray: ICandidateProfile[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const candidateProfileCollection: ICandidateProfile[] = [sampleWithRequiredData];
        expectedResult = service.addCandidateProfileToCollectionIfMissing(candidateProfileCollection, ...candidateProfileArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const candidateProfile: ICandidateProfile = sampleWithRequiredData;
        const candidateProfile2: ICandidateProfile = sampleWithPartialData;
        expectedResult = service.addCandidateProfileToCollectionIfMissing([], candidateProfile, candidateProfile2);
        expect(expectedResult).toEqual([candidateProfile, candidateProfile2]);
      });

      it('should accept null and undefined values', () => {
        const candidateProfile: ICandidateProfile = sampleWithRequiredData;
        expectedResult = service.addCandidateProfileToCollectionIfMissing([], null, candidateProfile, undefined);
        expect(expectedResult).toEqual([candidateProfile]);
      });

      it('should return initial array if no CandidateProfile is added', () => {
        const candidateProfileCollection: ICandidateProfile[] = [sampleWithRequiredData];
        expectedResult = service.addCandidateProfileToCollectionIfMissing(candidateProfileCollection, undefined, null);
        expect(expectedResult).toEqual(candidateProfileCollection);
      });
    });

    describe('compareCandidateProfile', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareCandidateProfile(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 25911 };
        const entity2 = null;

        const compareResult1 = service.compareCandidateProfile(entity1, entity2);
        const compareResult2 = service.compareCandidateProfile(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 25911 };
        const entity2 = { id: 10019 };

        const compareResult1 = service.compareCandidateProfile(entity1, entity2);
        const compareResult2 = service.compareCandidateProfile(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 25911 };
        const entity2 = { id: 25911 };

        const compareResult1 = service.compareCandidateProfile(entity1, entity2);
        const compareResult2 = service.compareCandidateProfile(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
